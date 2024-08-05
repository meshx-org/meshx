use std::rc::Rc;

use cm_types::Name;
use cm_config::SecurityPolicy;
use fiber_rust as fx;

/// Trait for built-in runner services. Wraps the generic Runner trait to provide a
/// ScopedPolicyChecker for the realm of the component being started, so that runners can enforce
/// security policy.
pub trait BuiltinRunnerFactory {
    /// Get a connection to a scoped runner by pipelining a
    /// `fuchsia.component.runner/ComponentRunner` server endpoint.
    fn get_scoped_runner(
        self: Rc<Self>,
        //checker: ScopedPolicyChecker,
        //open_request: OpenRequest<'_>,
    ) -> Result<(), fx::Status>;
}

/// Provides a hook for routing built-in runners to realms.
pub struct BuiltinRunner {
    name: Name,
    factory: Rc<dyn BuiltinRunnerFactory>,
    security_policy: Rc<SecurityPolicy>,
}

impl BuiltinRunner {
    pub fn new(name: Name, factory: Rc<dyn BuiltinRunnerFactory>, security_policy: Rc<SecurityPolicy>) -> Self {
        Self {
            name,
            factory,
            security_policy,
        }
    }
}
