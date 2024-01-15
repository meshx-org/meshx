use std::{cell::RefCell, rc::Rc, sync::Arc};

use crate::object::ProcessDispatcher;

thread_local! {
    static TL_SCOPES: RefCell<Vec<Context >> = RefCell::new(Vec::new())
}

#[derive(Clone)]
pub(crate) struct Context {
    pub process: Arc<ProcessDispatcher>,
}

impl std::fmt::Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context").field("process", &self.process).finish()
    }
}

pub(crate) struct ScopeGuard;

impl ScopeGuard {
    pub(crate) fn new(context: Context) -> Self {
        TL_SCOPES.with(|s: &RefCell<Vec<Context>>| {
            s.borrow_mut().push(context);
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

pub(crate) fn get_last_process() -> Context {
    TL_SCOPES.with_borrow(|scopes| match scopes.last() {
        Some(logger) => logger.clone(),
        None => panic!("No logger in scope"),
    })
}

/// Access the `Logger` for the current logging scope
///
/// This function doesn't have to clone the Logger
/// so it might be a bit faster.
pub(crate) fn with_context<F, R>(f: F) -> R
where
    F: FnOnce(&Context) -> R,
{
    TL_SCOPES.with(|scopes| {
        let scopes = scopes.borrow();

        match scopes.last() {
            Some(logger) => f(logger),
            None => panic!("No logger in scope"),
        }
    })
}
