mod runner;
mod future_hashmap;

use super::watcher;
use runner::Runner;
use crate::{
    reflector::{
        self, reflector,
        store::{Store, Writer},
        ObjectRef,
    },
    scheduler::{debounced_scheduler, ScheduleRequest},
    utils::{trystream_try_via, CancelableJoinHandle, KubeRuntimeStreamExt, StreamBackoff, WatchStreamExt},
    watcher::{watcher, DefaultBackoff},
};
use backoff::backoff::Backoff;
use controlplane_client::{core::dynamic::DynamicObject, Api, Resource};
use derivative::Derivative;
use futures::{
    channel,
    future::{self, BoxFuture},
    stream, FutureExt, Stream, StreamExt, TryFuture, TryFutureExt, TryStream, TryStreamExt,
};
use pin_project::pin_project;
use serde::de::DeserializeOwned;
use std::{
    fmt::{Debug, Display},
    future::Future,
    hash::Hash,
    sync::Arc,
    task::{ready, Poll},
    time::Duration,
};
use stream::BoxStream;
use thiserror::Error;
use tokio::{runtime::Handle, time::Instant};
use tracing::{info_span, Instrument};

pub type RunnerError = runner::Error<reflector::store::WriterDropped>;

#[derive(Debug, Error)]
pub enum Error<ReconcilerErr: 'static, QueueErr: 'static> {
    #[error("tried to reconcile object {0} that was not found in local store")]
    ObjectNotFound(ObjectRef<DynamicObject>),

    #[error("reconciler for object {1} failed")]
    ReconcilerFailed(#[source] ReconcilerErr, ObjectRef<DynamicObject>),

    #[error("event queue error")]
    QueueError(#[source] QueueErr),
    
    #[error("runner error")]
    RunnerError(#[source] RunnerError),
}

/// A request to reconcile an object, annotated with why that request was made.
///
/// NOTE: The reason is ignored for comparison purposes. This means that, for example,
/// an object can only occupy one scheduler slot, even if it has been scheduled for multiple reasons.
/// In this case, only *the first* reason is stored.
#[derive(Derivative)]
#[derivative(
    Debug(bound = "K::DynamicType: Debug"),
    Clone(bound = "K::DynamicType: Clone"),
    PartialEq(bound = "K::DynamicType: PartialEq"),
    Eq(bound = "K::DynamicType: Eq"),
    Hash(bound = "K::DynamicType: Hash")
)]
pub struct ReconcileRequest<K: Resource> {
    pub obj_ref: ObjectRef<K>,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub reason: ReconcileReason,
}

impl<K: Resource> From<ObjectRef<K>> for ReconcileRequest<K> {
    fn from(obj_ref: ObjectRef<K>) -> Self {
        ReconcileRequest {
            obj_ref,
            reason: ReconcileReason::Unknown,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ReconcileReason {
    Unknown,
    ObjectUpdated,
    RelatedObjectUpdated { obj_ref: Box<ObjectRef<DynamicObject>> },
    ReconcilerRequestedRetry,
    ErrorPolicyRequestedRetry,
    BulkReconcile,
    Custom { reason: String },
}

impl Display for ReconcileReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReconcileReason::Unknown => f.write_str("unknown"),
            ReconcileReason::ObjectUpdated => f.write_str("object updated"),
            ReconcileReason::RelatedObjectUpdated { obj_ref: object } => {
                f.write_fmt(format_args!("related object updated: {object}"))
            }
            ReconcileReason::BulkReconcile => f.write_str("bulk reconcile requested"),
            ReconcileReason::ReconcilerRequestedRetry => f.write_str("reconciler requested retry"),
            ReconcileReason::ErrorPolicyRequestedRetry => f.write_str("error policy requested retry"),
            ReconcileReason::Custom { reason } => f.write_str(reason),
        }
    }
}

/// Helper for building custom trigger filters, see the implementations of [`trigger_self`] and [`trigger_owners`] for some examples.
pub fn trigger_with<T, K, I, S>(
    stream: S,
    mapper: impl Fn(T) -> I,
) -> impl Stream<Item = Result<ReconcileRequest<K>, S::Error>>
where
    S: TryStream<Ok = T>,
    I: IntoIterator,
    I::Item: Into<ReconcileRequest<K>>,
    K: Resource,
{
    stream
        .map_ok(move |obj| stream::iter(mapper(obj).into_iter().map(Into::into).map(Ok)))
        .try_flatten()
}

/// Enqueues the object itself for reconciliation
pub fn trigger_self<K, S>(
    stream: S,
    dyntype: K::DynamicType,
) -> impl Stream<Item = Result<ReconcileRequest<K>, S::Error>>
where
    S: TryStream<Ok = K>,
    K: Resource,
    K::DynamicType: Clone,
{
    trigger_with(stream, move |obj| {
        Some(ReconcileRequest {
            obj_ref: ObjectRef::from_obj_with(&obj, dyntype.clone()),
            reason: ReconcileReason::ObjectUpdated,
        })
    })
}

const APPLIER_REQUEUE_BUF_SIZE: usize = 100;

/// Apply a reconciler to an input stream, with a given retry policy
///
/// Takes a `store` parameter for the core objects, which should usually be updated by a [`reflector()`].
///
/// The `queue` indicates which objects should be reconciled. For the core objects this will usually be
/// the [`reflector()`] (piped through [`trigger_self`]). If your core objects own any subobjects then you
/// can also make them trigger reconciliations by [merging](`futures::stream::select`) the [`reflector()`]
/// with a [`watcher()`] or [`reflector()`] for the subobject.
///
/// This is the "hard-mode" version of [`Controller`], which allows you some more customization
/// (such as triggering from arbitrary [`Stream`]s), at the cost of being a bit more verbose.
#[allow(clippy::needless_pass_by_value)]
pub fn applier<K, QueueStream, ReconcilerFut, Ctx>(
    mut reconciler: impl FnMut(Arc<K>, Arc<Ctx>) -> ReconcilerFut,
    error_policy: impl Fn(Arc<K>, &ReconcilerFut::Error, Arc<Ctx>) -> Action,
    context: Arc<Ctx>,
    store: Store<K>,
    queue: QueueStream,
    config: Config,
) -> impl Stream<Item = Result<(ObjectRef<K>, Action), Error<ReconcilerFut::Error, QueueStream::Error>>>
where
    K: Clone + Resource + 'static,
    K::DynamicType: Debug + Eq + Hash + Clone + Unpin,
    ReconcilerFut: TryFuture<Ok = Action> + Unpin,
    ReconcilerFut::Error: std::error::Error + 'static,
    QueueStream: TryStream,
    QueueStream::Ok: Into<ReconcileRequest<K>>,
    QueueStream::Error: std::error::Error + 'static,
{
    let (scheduler_shutdown_tx, scheduler_shutdown_rx) = channel::oneshot::channel();
    let (scheduler_tx, scheduler_rx) =
        channel::mpsc::channel::<ScheduleRequest<ReconcileRequest<K>>>(APPLIER_REQUEUE_BUF_SIZE);
    let error_policy = Arc::new(error_policy);
    let delay_store = store.clone();
    // Create a stream of ObjectRefs that need to be reconciled
    trystream_try_via(
        // input: stream combining scheduled tasks and user specified inputs event
        Box::pin(stream::select(
            // 1. inputs from users queue stream
            queue
                .map_err(Error::QueueError)
                .map_ok(|request| ScheduleRequest {
                    message: request.into(),
                    run_at: Instant::now(),
                })
                .on_complete(async move {
                    // On error: scheduler has already been shut down and there is nothing for us to do
                    let _ = scheduler_shutdown_tx.send(());
                    tracing::debug!("applier queue terminated, starting graceful shutdown")
                }),
            // 2. requests sent to scheduler_tx
            scheduler_rx
                .map(Ok)
                .take_until(scheduler_shutdown_rx)
                .on_complete(async { tracing::debug!("applier scheduler consumer terminated") }),
        )),
        // all the Oks from the select gets passed through the scheduler stream, and are then executed
        move |s| {
            Runner::new(
                debounced_scheduler(s, config.debounce),
                config.concurrency,
                move |request| {
                    let request = request.clone();
                    match store.get(&request.obj_ref) {
                        Some(obj) => {
                            let scheduler_tx = scheduler_tx.clone();
                            let error_policy_ctx = context.clone();
                            let error_policy = error_policy.clone();
                            let reconciler_span = info_span!(
                                "reconciling object",
                                "object.ref" = %request.obj_ref,
                                object.reason = %request.reason
                            );
                            reconciler_span
                                .in_scope(|| reconciler(Arc::clone(&obj), context.clone()))
                                .into_future()
                                .then(move |res| {
                                    let error_policy = error_policy;
                                    RescheduleReconciliation::new(
                                        res,
                                        |err| error_policy(obj, err, error_policy_ctx),
                                        request.obj_ref.clone(),
                                        scheduler_tx,
                                    )
                                    // Reconciler errors are OK from the applier's PoV, we need to apply the error policy
                                    // to them separately
                                    .map(|res| Ok((request.obj_ref, res)))
                                })
                                .instrument(reconciler_span)
                                .left_future()
                        }
                        None => std::future::ready(Err(Error::ObjectNotFound(request.obj_ref.erase()))).right_future(),
                    }
                },
            )
            .delay_tasks_until(async move {
                tracing::debug!("applier runner held until store is ready");
                let res = delay_store.wait_until_ready().await;
                tracing::debug!("store is ready, starting runner");
                res
            })
            .map(|runner_res| runner_res.unwrap_or_else(|err| Err(Error::RunnerError(err))))
            .on_complete(async { tracing::debug!("applier runner terminated") })
        },
    )
    .on_complete(async { tracing::debug!("applier runner-merge terminated") })
    // finally, for each completed reconcile call:
    .and_then(move |(obj_ref, reconciler_result)| async move {
        match reconciler_result {
            Ok(action) => Ok((obj_ref, action)),
            Err(err) => Err(Error::ReconcilerFailed(err, obj_ref.erase())),
        }
    })
    .on_complete(async { tracing::debug!("applier terminated") })
}

/// Internal helper [`Future`] that reschedules reconciliation of objects (if required), in the scheduled context of the reconciler
///
/// This could be an `async fn`, but isn't because we want it to be [`Unpin`]
#[pin_project]
#[must_use]
struct RescheduleReconciliation<K: Resource, ReconcilerErr> {
    reschedule_tx: channel::mpsc::Sender<ScheduleRequest<ReconcileRequest<K>>>,

    reschedule_request: Option<ScheduleRequest<ReconcileRequest<K>>>,
    result: Option<Result<Action, ReconcilerErr>>,
}

impl<K, ReconcilerErr> RescheduleReconciliation<K, ReconcilerErr>
where
    K: Resource,
{
    fn new(
        result: Result<Action, ReconcilerErr>,
        error_policy: impl FnOnce(&ReconcilerErr) -> Action,
        obj_ref: ObjectRef<K>,
        reschedule_tx: channel::mpsc::Sender<ScheduleRequest<ReconcileRequest<K>>>,
    ) -> Self {
        let reconciler_finished_at = Instant::now();

        let (action, reschedule_reason) = result.as_ref().map_or_else(
            |err| (error_policy(err), ReconcileReason::ErrorPolicyRequestedRetry),
            |action| (action.clone(), ReconcileReason::ReconcilerRequestedRetry),
        );

        Self {
            reschedule_tx,
            reschedule_request: action.requeue_after.map(|requeue_after| ScheduleRequest {
                message: ReconcileRequest {
                    obj_ref,
                    reason: reschedule_reason,
                },
                run_at: reconciler_finished_at
                    .checked_add(requeue_after)
                    .unwrap_or_else(crate::scheduler::far_future),
            }),
            result: Some(result),
        }
    }
}

impl<K, ReconcilerErr> Future for RescheduleReconciliation<K, ReconcilerErr>
where
    K: Resource,
{
    type Output = Result<Action, ReconcilerErr>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        if this.reschedule_request.is_some() {
            let rescheduler_ready = ready!(this.reschedule_tx.poll_ready(cx));
            let reschedule_request = this
                .reschedule_request
                .take()
                .expect("PostReconciler::reschedule_request was taken during processing");
            // Failure to schedule item = in graceful shutdown mode, ignore
            if let Ok(()) = rescheduler_ready {
                let _ = this.reschedule_tx.start_send(reschedule_request);
            }
        }

        Poll::Ready(this.result.take().expect("PostReconciler::result was already taken"))
    }
}

/// Results of the reconciliation attempt
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Action {
    /// Whether (and when) to next trigger the reconciliation if no external watch triggers hit
    ///
    /// For example, use this to query external systems for updates, expire time-limited resources, or
    /// (in your `error_policy`) retry after errors.
    requeue_after: Option<Duration>,
}

impl Action {
    /// Action to the reconciliation at this time even if no external watch triggers hit
    ///
    /// This is the best-practice action that ensures eventual consistency of your controller
    /// even in the case of missed changes (which can happen).
    ///
    /// Watch events are not normally missed, so running this once per hour (`Default`) as a fallback is reasonable.
    #[must_use]
    pub fn requeue(duration: Duration) -> Self {
        Self {
            requeue_after: Some(duration),
        }
    }

    /// Do nothing until a change is detected
    ///
    /// This stops the controller periodically reconciling this object until a relevant watch event
    /// was **detected**.
    ///
    /// **Warning**: If you have watch desyncs, it is possible to miss changes entirely.
    /// It is therefore not recommended to disable requeuing this way, unless you have
    /// frequent changes to the underlying object, or some other hook to retain eventual consistency.
    #[must_use]
    pub fn await_change() -> Self {
        Self { requeue_after: None }
    }
}

/// Accumulates all options that can be used on a [`Controller`] invocation.
#[derive(Clone, Debug, Default)]
pub struct Config {
    debounce: Duration,
    concurrency: u16,
}

impl Config {
    /// The debounce duration used to deduplicate reconciliation requests.
    ///
    /// When set to a non-zero duration, debouncing is enabled in the [`scheduler`](crate::scheduler())
    /// resulting in __trailing edge debouncing__ of reconciler requests.
    /// This option can help to reduce the amount of unnecessary reconciler calls
    /// when using multiple controller relations, or during rapid phase transitions.
    ///
    /// ## Warning
    /// This option delays (and keeps delaying) reconcile requests for objects while
    /// the object is updated. It can **permanently hide** updates from your reconciler
    /// if set too high on objects that are updated frequently (like nodes).
    #[must_use]
    pub fn debounce(mut self, debounce: Duration) -> Self {
        self.debounce = debounce;
        self
    }

    /// The number of concurrent reconciliations of that are allowed to run at an given moment.
    ///
    /// This can be adjusted to the controller's needs to increase
    /// performance and/or make performance predictable. By default, its 0 meaning
    /// the controller runs with unbounded concurrency.
    ///
    /// Note that despite concurrency, a controller never schedules concurrent reconciles
    /// on the same object.
    #[must_use]
    pub fn concurrency(mut self, concurrency: u16) -> Self {
        self.concurrency = concurrency;
        self
    }
}

/// Controller for a Resource `K`
///
/// A controller is an infinite stream of objects to be reconciled.
///
/// Once `run` and continuously awaited, it continuously calls out to user provided
/// `reconcile` and `error_policy` callbacks whenever relevant changes are detected
/// or if errors are seen from `reconcile`.
///
/// Reconciles are generally requested for all changes on your root objects.
/// Changes to managed child resources will also trigger the reconciler for the
/// managing object by traversing owner references (for `Controller::owns`),
/// or traverse a custom mapping (for `Controller::watches`).
///
/// This mapping mechanism ultimately hides the reason for the reconciliation request,
/// and forces you to write an idempotent reconciler.
///
/// General setup:
/// ```no_run
/// use kube::{Api, Client, CustomResource};
/// use kube::runtime::{controller::{Controller, Action}, watcher};
/// # use serde::{Deserialize, Serialize};
/// # use tokio::time::Duration;
/// use futures::StreamExt;
/// use k8s_openapi::api::core::v1::ConfigMap;
/// use schemars::JsonSchema;
/// # use std::sync::Arc;
/// use thiserror::Error;
///
/// #[derive(Debug, Error)]
/// enum Error {}
///
/// /// A custom resource
/// #[derive(CustomResource, Debug, Clone, Deserialize, Serialize, JsonSchema)]
/// #[kube(group = "nullable.se", version = "v1", kind = "ConfigMapGenerator", namespaced)]
/// struct ConfigMapGeneratorSpec {
///     content: String,
/// }
///
/// /// The reconciler that will be called when either object change
/// async fn reconcile(g: Arc<ConfigMapGenerator>, _ctx: Arc<()>) -> Result<Action, Error> {
///     // .. use api here to reconcile a child ConfigMap with ownerreferences
///     // see configmapgen_controller example for full info
///     Ok(Action::requeue(Duration::from_secs(300)))
/// }
/// /// an error handler that will be called when the reconciler fails with access to both the
/// /// object that caused the failure and the actual error
/// fn error_policy(obj: Arc<ConfigMapGenerator>, _error: &Error, _ctx: Arc<()>) -> Action {
///     Action::requeue(Duration::from_secs(60))
/// }
///
/// /// something to drive the controller
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = Client::try_default().await?;
///     let context = Arc::new(()); // bad empty context - put client in here
///     let cmgs = Api::<ConfigMapGenerator>::all(client.clone());
///     let cms = Api::<ConfigMap>::all(client.clone());
///     Controller::new(cmgs, watcher::Config::default())
///         .owns(cms, watcher::Config::default())
///         .run(reconcile, error_policy, context)
///         .for_each(|res| async move {
///             match res {
///                 Ok(o) => println!("reconciled {:?}", o),
///                 Err(e) => println!("reconcile failed: {:?}", e),
///             }
///         })
///         .await; // controller does nothing unless polled
///     Ok(())
/// }
/// ```
pub struct Controller<K>
where
    K: Clone + Resource + Debug + 'static,
    K::DynamicType: Eq + Hash,
{
    // NB: Need to Unpin for stream::select_all
    trigger_selector: stream::SelectAll<BoxStream<'static, Result<ReconcileRequest<K>, watcher::Error>>>,
    trigger_backoff: Box<dyn Backoff + Send>,
    /// [`run`](crate::Controller::run) starts a graceful shutdown when any of these [`Future`]s complete,
    /// refusing to start any new reconciliations but letting any existing ones finish.
    graceful_shutdown_selector: Vec<BoxFuture<'static, ()>>,
    /// [`run`](crate::Controller::run) terminates immediately when any of these [`Future`]s complete,
    /// requesting that all running reconciliations be aborted.
    /// However, note that they *will* keep running until their next yield point (`.await`),
    /// blocking [`tokio::runtime::Runtime`] destruction (unless you follow up by calling [`std::process::exit`] after `run`).
    forceful_shutdown_selector: Vec<BoxFuture<'static, ()>>,
    dyntype: K::DynamicType,
    reader: Store<K>,
    config: Config,
}

impl<K> Controller<K>
where
    K: Clone + Resource + DeserializeOwned + Debug + Send + Sync + 'static,
    K::DynamicType: Eq + Hash + Clone,
{
    /// Create a Controller for a resource `K`
    ///
    /// Takes an [`Api`] object that determines how the `Controller` listens for changes to the `K`.
    ///
    /// The [`watcher::Config`] controls to the possible subset of objects of `K` that you want to manage
    /// and receive reconcile events for.
    /// For the full set of objects `K` in the given `Api` scope, you can use [`watcher::Config::default`].
    #[must_use]
    pub fn new(main_api: Api<K>, wc: watcher::Config) -> Self
    where
        K::DynamicType: Default,
    {
        Self::new_with(main_api, wc, Default::default())
    }

    /// Create a Controller for a resource `K`
    ///
    /// Takes an [`Api`] object that determines how the `Controller` listens for changes to the `K`.
    ///
    /// The [`watcher::Config`] lets you define a possible subset of objects of `K` that you want the [`Api`]
    /// to watch - in the Api's  configured scope - and receive reconcile events for.
    /// For the full set of objects `K` in the given `Api` scope, you can use [`Config::default`].
    ///
    /// This variant constructor is for [`dynamic`] types found through discovery. Prefer [`Controller::new`] for static types.
    ///
    /// [`watcher::Config`]: crate::watcher::Config
    /// [`Api`]: kube_client::Api
    /// [`dynamic`]: kube_client::core::dynamic
    /// [`Config::default`]: crate::watcher::Config::default
    pub fn new_with(main_api: Api<K>, wc: watcher::Config, dyntype: K::DynamicType) -> Self {
        let writer = Writer::<K>::new(dyntype.clone());
        let reader = writer.as_reader();
        let mut trigger_selector = stream::SelectAll::new();

        let self_watcher = trigger_self(
            reflector(writer, watcher(main_api, wc)).applied_objects(),
            dyntype.clone(),
        )
        .boxed();

        trigger_selector.push(self_watcher);

        Self {
            trigger_selector,
            trigger_backoff: Box::<DefaultBackoff>::default(),
            graceful_shutdown_selector: vec![
                // Fallback future, ensuring that we never terminate if no additional futures are added to the selector
                future::pending().boxed(),
            ],
            forceful_shutdown_selector: vec![
                // Fallback future, ensuring that we never terminate if no additional futures are added to the selector
                future::pending().boxed(),
            ],
            dyntype,
            reader,
            config: Default::default(),
        }
    }

    /// Initiate graceful shutdown on Ctrl+C or SIGTERM (on Unix), waiting for all reconcilers to finish.
    ///
    /// Once a graceful shutdown has been initiated, Ctrl+C (or SIGTERM) can be sent again
    /// to request a forceful shutdown (requesting that all reconcilers abort on the next yield point).
    ///
    /// NOTE: On Unix this leaves the default handlers for SIGINT and SIGTERM disabled after the [`Controller`] has
    /// terminated. If you run this in a process containing more tasks than just the [`Controller`], ensure that
    /// all other tasks either terminate when the [`Controller`] does, that they have their own signal handlers,
    /// or use [`Controller::graceful_shutdown_on`] to manage your own shutdown strategy.
    ///
    /// NOTE: If developing a Windows service then you need to listen to its lifecycle events instead, and hook that into
    /// [`Controller::graceful_shutdown_on`].
    ///
    /// NOTE: [`Controller::run`] terminates as soon as a forceful shutdown is requested, but leaves the reconcilers running
    /// in the background while they terminate. This will block [`tokio::runtime::Runtime`] termination until they actually terminate,
    /// unless you run [`std::process::exit`] afterwards.
    #[must_use]
    pub fn shutdown_on_signal(mut self) -> Self {
        async fn shutdown_signal() {
            futures::future::select(
                tokio::signal::ctrl_c().map(|_| ()).boxed(),
                #[cfg(unix)]
                tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                    .unwrap()
                    .recv()
                    .map(|_| ())
                    .boxed(),
                // Assume that ctrl_c is enough on non-Unix platforms (such as Windows)
                #[cfg(not(unix))]
                futures::future::pending::<()>(),
            )
            .await;
        }

        let (graceful_tx, graceful_rx) = channel::oneshot::channel();
        self.graceful_shutdown_selector.push(graceful_rx.map(|_| ()).boxed());
        self.forceful_shutdown_selector.push(
            async {
                tracing::info!("press ctrl+c to shut down gracefully");
                shutdown_signal().await;
                if let Ok(()) = graceful_tx.send(()) {
                    tracing::info!("graceful shutdown requested, press ctrl+c again to force shutdown");
                } else {
                    tracing::info!("graceful shutdown already requested, press ctrl+c again to force shutdown");
                }
                shutdown_signal().await;
                tracing::info!("forced shutdown requested");
            }
            .boxed(),
        );
        self
    }

    /// Consume all the parameters of the Controller and start the applier stream
    ///
    /// This creates a stream from all builder calls and starts an applier with
    /// a specified `reconciler` and `error_policy` callbacks. Each of these will be called
    /// with a configurable `context`.
    pub fn run<ReconcilerFut, Ctx>(
        self,
        mut reconciler: impl FnMut(Arc<K>, Arc<Ctx>) -> ReconcilerFut,
        error_policy: impl Fn(Arc<K>, &ReconcilerFut::Error, Arc<Ctx>) -> Action,
        context: Arc<Ctx>,
    ) -> impl Stream<Item = Result<(ObjectRef<K>, Action), Error<ReconcilerFut::Error, watcher::Error>>>
    where
        K::DynamicType: Debug + Unpin,
        ReconcilerFut: TryFuture<Ok = Action> + Send + 'static,
        ReconcilerFut::Error: std::error::Error + Send + 'static,
    {
        applier(
            move |obj, ctx| {
                CancelableJoinHandle::spawn(reconciler(obj, ctx).into_future().in_current_span(), &Handle::current())
            },
            error_policy,
            context,
            self.reader,
            StreamBackoff::new(self.trigger_selector, self.trigger_backoff)
                .take_until(future::select_all(self.graceful_shutdown_selector)),
            self.config,
        )
        .take_until(futures::future::select_all(self.forceful_shutdown_selector))
    }
}
