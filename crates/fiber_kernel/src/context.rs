use std::cell::RefCell;

thread_local! {
    static TL_SCOPES: RefCell<Vec<*const Context>> = RefCell::new(Vec::with_capacity(8))
}

#[derive(Debug)]
pub struct Context {
    pub test: String
}

impl Context {}

pub struct ScopeGuard;

impl ScopeGuard {
    pub fn new(context: &Context) -> Self {
        TL_SCOPES.with(|s| {
            s.borrow_mut().push(context as *const Context);
        });

        ScopeGuard
    }
}

impl Drop for ScopeGuard {
    fn drop(&mut self) {
        TL_SCOPES.with(|s| {
            s.borrow_mut().pop().expect("TL_SCOPES should contain a logger");
        })
    }
}

/// Access the `Logger` for the current logging scope
///
/// This function doesn't have to clone the Logger
/// so it might be a bit faster.
pub fn with_logger<F, R>(f: F) -> R
where
    F: FnOnce(&Context) -> R,
{
    TL_SCOPES.with(|s| {
        let s = s.borrow();
        match s.last() {
            Some(logger) => f(unsafe { &**logger }),
            None => panic!("No logger in scope"),
        }
    })
}

/// Execute code in a logging scope
///
/// Logging scopes allow using a `slog::Logger` without explicitly
/// passing it in the code.
///
/// At any time current active `Logger` for a given thread can be retrived
/// with `logger()` call.
///
/// Logging scopes can be nested and are panic safe.
///
/// `logger` is the `Logger` to use during the duration of `f`.
/// `with_current_logger` can be used to build it as a child of currently active
/// logger.
///
/// `f` is a code to be executed in the logging scope.
///
/// Note: Thread scopes are thread-local. Each newly spawned thread starts
/// with a global logger, as a current logger.
pub fn scope<SF, R>(logger: &Context, f: SF) -> R
where
    SF: FnOnce() -> R,
{
    let _guard = ScopeGuard::new(&logger);
    f()
}
