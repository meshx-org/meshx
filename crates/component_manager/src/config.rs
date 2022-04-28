// Copyright 2020 The Fuchsia Authors. All rights reserved.

use crate::types::{CapabilityName, CapabilityTypeName};
use std::collections::{HashMap, HashSet};
use moniker::{AbsoluteMoniker, ExtendedMoniker};

/// Runtime configuration options.
/// This configuration intended to be "global", in that the same configuration
/// is applied throughout a given running instance of component_manager.
#[derive(Debug, PartialEq, Eq)]
pub struct RuntimeConfig {
    /// How many children, maximum, are returned by a call to `ChildIterator.next()`.
    pub list_children_batch_size: usize,

    /// Security policy configuration.
    pub security_policy: SecurityPolicy,

    /// If true, component manager will be in debug mode. In this mode, component manager
    /// provides the `EventSource` protocol and exposes this protocol. The root component
    /// must be manually started using the LifecycleController protocol in the hub.
    ///
    /// This is done so that an external component (say an integration test) can subscribe
    /// to events before the root component has started.
    pub debug: bool,

    /// If true, component_manager will serve an instance of fuchsia.process.Launcher and use this
    /// launcher for the built-in ELF component runner. The root component can additionally
    /// use and/or offer this service using '/builtin/fuchsia.process.Launcher' from realm.
    // This flag exists because the built-in process launcher *only* works when
    // component_manager runs under a job that has ZX_POL_NEW_PROCESS set to allow, like the root
    // job. Otherwise, the component_manager process cannot directly create process through
    // zx_process_create. When we run component_manager elsewhere, like in test environments, it
    // has to use the fuchsia.process.Launcher service provided through its namespace instead.
    pub use_builtin_process_launcher: bool,

    // The number of threads to use for running component_manager's executor.
    // Value defaults to 1.
    pub num_threads: usize,

    /// The list of capabilities offered from component manager's namespace.
    // pub namespace_capabilities: Vec<cm_rust::CapabilityDecl>,

    /// The list of capabilities offered from component manager as built-in capabilities.
    // pub builtin_capabilities: Vec<cm_rust::CapabilityDecl>,

    /// Which builtin resolver to use for the fuchsia-pkg scheme. If not supplied this defaults to
    /// the NONE option.
    // pub builtin_pkg_resolver: BuiltinPkgResolver,

    /// Determine what content to expose through the component manager's
    /// outgoing directory.
    // pub out_dir_contents: OutDirContents,

    /// URL of the root component to launch. This field is used if no URL
    /// is passed to component manager. If value is passed in both places, then
    /// an error is raised.
    // pub root_component_url: Option<Url>,

    /// Path to the component ID index, parsed from
    /// `fuchsia.component.internal.RuntimeConfig.component_id_index_path`.
    pub component_id_index_path: Option<String>,

    /// Where to log to.
    // pub log_destination: LogDestination,

    /// If true, component manager will log all events dispatched in the topology.
    pub log_all_events: bool,

    /// Which builtin resolver to use for the fuchsia-boot scheme. If not supplied this defaults to
    /// the NONE option.
    //pub builtin_boot_resolver: BuiltinBootResolver,

    /// If true, allow components to set the `OnTerminate=REBOOT` option.
    ///
    /// This lets a parent component designate that the system should reboot if a child terminates
    /// (except when it's shut down).
    pub reboot_on_terminate_enabled: bool,

    // If and how the realm builder resolver and runner are enabled.
    //pub realm_builder_resolver_and_runner: RealmBuilderResolverAndRunner,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            list_children_batch_size: 1000,
            // security_policy must default to empty to ensure that it fails closed if no
            // configuration is present or it fails to load.
            security_policy: Default::default(),
            debug: false,
            use_builtin_process_launcher: false,
            num_threads: 1,
            //namespace_capabilities: vec![],
            //builtin_capabilities: vec![],
            //builtin_pkg_resolver: BuiltinPkgResolver::None,
            //out_dir_contents: OutDirContents::None,
            //root_component_url: Default::default(),
            component_id_index_path: None,
            //log_destination: LogDestination::Syslog,
            log_all_events: false,
            //builtin_boot_resolver: BuiltinBootResolver::None,
            reboot_on_terminate_enabled: false,
            //realm_builder_resolver_and_runner: RealmBuilderResolverAndRunner::None,
        }
    }
}

/// A single security policy allowlist entry.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum AllowlistEntry {
    /// Allow the component with this exact AbsoluteMoniker.
    /// Example string form in config: "/foo/bar", "/foo/bar/baz"
    Exact(AbsoluteMoniker),
    /// Allow any components that are children of this AbsoluteMoniker. In other words, a
    /// prefix match against the target moniker.
    /// Example string form in config: "/foo/**", "/foo/bar/**"
    Realm(AbsoluteMoniker),
    /// Allow any components that are in AbsoluteMoniker's collection with the given name.
    /// Also a prefix match against the target moniker but additionally scoped to a specific
    /// collection.
    /// Example string form in config: "/foo/tests:**", "/bootstrap/drivers:**"
    Collection(AbsoluteMoniker, String),
}

/// Allowlists for child option policy. Part of runtime security policy.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct ChildPolicyAllowlists {
    /// Absolute monikers of component instances allowed to have the
    /// `on_terminate=REBOOT` in their `children` declaration.
    pub reboot_on_terminate: Vec<AllowlistEntry>,
}

/// The available capability sources for capability allow lists. This is a strict
/// subset of all possible Ref types, with equality support.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum CapabilityAllowlistSource {
    Self_,
    Framework,
    Capability,
}

/// Allowlist key for capability routing policy. Part of the runtime
/// security policy. This defines all the required keying information to lookup
/// whether a capability exists in the policy map or not.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct CapabilityAllowlistKey {
    pub source_moniker: ExtendedMoniker,
    pub source_name: CapabilityName,
    pub source: CapabilityAllowlistSource,
    pub capability: CapabilityTypeName,
}

/// Runtime security policy.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct SecurityPolicy {
    /// Capability routing policies. The key contains all the information required
    /// to uniquely identify any routable capability and the set of monikers
    /// define the set of component paths that are allowed to access this specific
    /// capability.
    pub capability_policy: HashMap<CapabilityAllowlistKey, HashSet<AllowlistEntry>>,

    /// Debug Capability routing policies. The key contains all the information required
    /// to uniquely identify any routable capability and the set of (monikers, environment_name)
    /// define the set of components which were allowed to register it as a debug capability in
    /// their environment `environment_name`.
    pub debug_capability_policy:
        HashMap<CapabilityAllowlistKey, HashSet<(AbsoluteMoniker, String)>>,

    /// Allowlists component child policy. These allowlists control what components are allowed
    /// to set privileged options on their children.
    pub child_policy: ChildPolicyAllowlists,
}
