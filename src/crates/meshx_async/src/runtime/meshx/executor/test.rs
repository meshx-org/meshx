/// A single-threaded executor for testing. Exposes additional APIs for manipulating executor state
/// and validating behavior of executed tasks.
pub struct TestExecutor {
    /// LocalExecutor used under the hood, since most of the logic is shared.
    local: LocalExecutor,
    // A packet that has been dequeued but not processed. This is used by `run_one_step`.
    next_packet: Option<zx::Packet>,
}

impl TestExecutor {
    /// Create a new executor for testing.
    pub fn new() -> Self {
        Self {
            local: LocalExecutor::new(),
            next_packet: None,
        }
    }

    /// Create a new single-threaded executor running with fake time.
    pub fn new_with_fake_time() -> Self {
        let inner = Arc::new(Inner::new(
            ExecutorTime::FakeTime(AtomicI64::new(Time::INFINITE_PAST.into_nanos())),
            /* is_local */ true,
        ));
        inner.clone().set_local(TimerHeap::default());

        let main_task = Arc::new(MainTask {
            executor: Arc::downgrade(&inner),
            notifier: Notifier::default(),
        });

        Self {
            local: LocalExecutor {
                inner,
                main_task,
                main_waker: futures::task::waker(main_task.clone()),
            },
            next_packet: None,
        }
    }
}
