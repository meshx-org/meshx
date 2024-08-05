use anyhow::anyhow;
use anyhow::format_err;
use anyhow::Error;
use cm_config::RuntimeConfig;
use cm_types::Name;
use std::rc::Rc;

use crate::builtin::runner::{BuiltinRunner, BuiltinRunnerFactory};
use crate::model::component::manager::ComponentManagerInstance;
use crate::model::token::InstanceRegistry;

pub struct BuiltinEnvironmentBuilder {
    // TODO(60804): Make component manager's namespace injectable here.
    runtime_config: Option<RuntimeConfig>,
    top_instance: Option<Rc<ComponentManagerInstance>>,
    //bootfs_svc: Option<BootfsSvc>,
    runners: Vec<(Name, Rc<dyn BuiltinRunnerFactory>)>,
    //resolvers: ResolverRegistry,
    //utc_clock: Option<Arc<Clock>>,
    add_environment_resolvers: bool,
    //inspector: Option<Inspector>,
    //crash_records: CrashRecords,
    instance_registry: Rc<InstanceRegistry>,
}

impl Default for BuiltinEnvironmentBuilder {
    fn default() -> Self {
        Self {
            runtime_config: None,
            top_instance: None,
            //bootfs_svc: None,
            runners: vec![],
            //resolvers: ResolverRegistry::default(),
            //utc_clock: None,
            add_environment_resolvers: false,
            //inspector: None,
            //crash_records: CrashRecords::new(),
            instance_registry: InstanceRegistry::new(),
        }
    }
}

impl BuiltinEnvironmentBuilder {
    pub fn new() -> Self {
        BuiltinEnvironmentBuilder::default()
    }

    pub fn set_runtime_config(mut self, runtime_config: RuntimeConfig) -> Self {
        assert!(self.runtime_config.is_none());
        let top_instance = Rc::new(ComponentManagerInstance::new(
           // runtime_config.namespace_capabilities.clone(),
           // runtime_config.builtin_capabilities.clone(),
        ));
        self.runtime_config = Some(runtime_config);
        self.top_instance = Some(top_instance);
        self
    }

    pub fn add_builtin_runner(self) -> Result<Self, Error> {
        use crate::builtin::builtin_runner::{BuiltinRunner, WasmRunnerResources};

        let runtime_config = self
            .runtime_config
            .as_ref()
            .ok_or(format_err!("Runtime config should be set to add builtin runner."))?;

        let top_instance = self.top_instance.clone().unwrap();
        let runner = Rc::new(BuiltinRunner::new(
            //top_instance.task_group(),
            WasmRunnerResources {
                security_policy: runtime_config.security_policy.clone(),
                // utc_clock: self.utc_clock.clone(),
                // crash_records: self.crash_records.clone(),
                instance_registry: self.instance_registry.clone(),
            },
        ));

        Ok(self.add_runner("builtin".parse().unwrap(), runner))
    }

    /// Adds standard resolvers whose dependencies are available in the process's namespace and for
    /// whose scheme no resolver is registered through `add_resolver` by the time `build()` is
    /// is called. This includes:
    ///   - A fuchsia-boot resolver if /boot is available.
    pub fn include_namespace_resolvers(mut self) -> Self {
        self.add_environment_resolvers = true;
        self
    }

    pub fn add_runner(mut self, name: Name, runner: Rc<dyn BuiltinRunnerFactory>) -> Self {
        // We don't wrap these in a BuiltinRunner immediately because that requires the
        // RuntimeConfig, which may be provided after this or may fall back to the default.
        self.runners.push((name, runner));
        self
    }

    #[cfg(test)]
    pub fn add_resolver(mut self, scheme: String, resolver: Box<dyn Resolver + Send + Sync + 'static>) -> Self {
        self.resolvers.register(scheme, resolver);
        self
    }

    pub async fn build(mut self) -> Result<BuiltinEnvironment, Error> {
        let runtime_config = self
            .runtime_config
            .ok_or(format_err!("Runtime config is required for BuiltinEnvironment."))?;

        let root_component_url = match runtime_config.root_component_url.as_ref() {
            Some(url) => url.clone(),
            None => {
                return Err(format_err!("Root component url is required from RuntimeConfig."));
            }
        };

        // Wrap BuiltinRunnerFactory in BuiltinRunner now that we have the definite RuntimeConfig.
        let builtin_runners = self
            .runners
            .into_iter()
            .map(|(name, runner)| BuiltinRunner::new(name, runner, runtime_config.security_policy.clone()))
            .collect();

        let runtime_config = Rc::new(runtime_config);

        Ok(BuiltinEnvironment::new(
            //params,
            runtime_config,
            //system_resource_handle,
            builtin_runners,
            false
            //boot_resolver,
            //realm_builder_resolver,
            //self.utc_clock,
            //self.inspector.unwrap_or(component::init_inspector_with_size(INSPECTOR_SIZE).clone()),
            //self.crash_records,
        )
        .await?)
    }
}

/// The built-in environment consists of the set of the root services and framework services. Use
/// BuiltinEnvironmentBuilder to construct one.
///
/// The available built-in capabilities depends on the configuration provided in Arguments:
/// * If [RuntimeConfig::use_builtin_process_launcher] is true, a fuchsia.process.Launcher service
///   is available.
/// * If [RuntimeConfig::maintain_utc_clock] is true, a fuchsia.time.Maintenance service is
///   available.
pub struct BuiltinEnvironment {
    pub debug: bool,
}

impl BuiltinEnvironment {
    async fn new(
        //arams: ModelParams,
        runtime_config: Rc<RuntimeConfig>,
        //system_resource_handle: Option<Resource>,
        builtin_runners: Vec<BuiltinRunner>,
        //boot_resolver: Option<FuchsiaBootResolver>,
        //realm_builder_resolver: Option<RealmBuilderResolver>,
        //utc_clock: Option<Arc<Clock>>,
        //inspector: Inspector,
        //crash_records: CrashRecords,
        capability_passthrough: bool,
    ) -> Result<BuiltinEnvironment, Error> {
        let debug = runtime_config.debug;

        Ok(BuiltinEnvironment {
            // model,
            // realm_query,
            // lifecycle_controller,
            // event_registry,
            // event_source_factory,
            // capability_store,
            // stop_notifier,
            // inspect_sink_provider,
            // event_stream_provider,
            // event_logger,
            // component_tree_stats,
            // _component_lifecycle_time_stats: component_lifecycle_time_stats,
            // _component_escrow_duration_status: component_escrow_duration_status,
            debug,
            //num_threads,
            // realm_builder_resolver,
            // capability_passthrough,
            //_service_fs_task: None,
        })
    }
}
