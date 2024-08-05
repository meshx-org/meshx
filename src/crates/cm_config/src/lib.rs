// Copyright 2020 The Fuchsia Authors. All rights reserved.

use cm_types::{CapabilityName, CapabilityTypeName};
use cm_types::{LongName, Name, Path, Url};
use moniker::{AbsoluteMoniker, ExtendedMoniker};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

//#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
//#[fidl_decl(fidl_table = "fdecl::Resolver")]
pub struct ResolverDecl {
    pub name: Name,
    pub source_path: Option<Path>,
}

//#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
//#[fidl_decl(fidl_table = "fdecl::Runner")]
pub struct RunnerDecl {
    pub name: Name,
    pub source_path: Option<Path>,
}

//#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
//#[fidl_decl(fidl_table = "fdecl::Protocol")]
pub struct ProtocolDecl {
    pub name: Name,
    pub source_path: Option<Path>,
    #[fidl_decl(default)]
    #[cfg(fuchsia_api_level_at_least = "HEAD")]
    pub delivery: DeliveryType,
}

//#[serde(tag = "type", rename_all = "snake_case")]
#[derive(Debug, Clone, PartialEq, Eq)]
//#[fidl_decl(fidl_union = "fdecl::Capability")]
pub enum CapabilityDecl {
    Protocol(ProtocolDecl),
    Runner(RunnerDecl),
    Resolver(ResolverDecl),
}

/// The builtin resolver to use for the fuchsia-boot scheme, if any.
#[derive(Debug, PartialEq, Eq)]
pub enum BuiltinBootResolver {
    /// No builtin boot resolver is used.
    None = 1,

    /// Try to use the /boot directory from the namespace. Typically this is provided
    /// to component manager during initialization of the system.
    Boot = 2,
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
    pub debug_capability_policy: HashMap<CapabilityAllowlistKey, HashSet<(AbsoluteMoniker, String)>>,

    /// Allowlists component child policy. These allowlists control what components are allowed
    /// to set privileged options on their children.
    pub child_policy: ChildPolicyAllowlists,
}

/// Runtime configuration options.
/// This configuration intended to be "global", in that the same configuration
/// is applied throughout a given running instance of component_manager.
#[derive(Debug, PartialEq, Eq)]
pub struct RuntimeConfig {
    /// How many children, maximum, are returned by a call to `ChildIterator.next()`.
    pub list_children_batch_size: usize,

    /// Security policy configuration.
    pub security_policy: Rc<SecurityPolicy>,

    /// If true, component manager will be in debug mode. In this mode, component manager
    /// provides the `EventSource` protocol and exposes this protocol. The root component
    /// must be manually started using the LifecycleController protocol in the hub.
    ///
    /// This is done so that an external component (say an integration test) can subscribe
    /// to events before the root component has started.
    pub debug: bool,

    /// The list of capabilities offered from component manager as built-in capabilities.
    pub builtin_capabilities: Vec<CapabilityDecl>,

    /// If true, component_manager will maintain a UTC kernel clock and vend write handles through
    /// an instance of `fuchsia.time.Maintenance`. This flag should only be used with the top-level
    /// component_manager.
    pub maintain_utc_clock: bool,

    /// Which builtin resolver to use for the fuchsia-boot scheme. If not supplied this defaults to
    /// the NONE option.
    pub builtin_boot_resolver: BuiltinBootResolver,

    /// URL of the root component to launch. This field is used if no URL
    /// is passed to component manager. If value is passed in both places, then
    /// an error is raised.
    pub root_component_url: Option<cm_types::Url>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            list_children_batch_size: 1000,
            // security_policy must default to empty to ensure that it fails closed if no
            // configuration is present or it fails to load.
            security_policy: Default::default(),
            debug: false,
            //enable_introspection: false,
            //use_builtin_process_launcher: false,
            maintain_utc_clock: false,
            //num_threads: 1,
            //namespace_capabilities: vec![],
            builtin_capabilities: vec![],
            root_component_url: Default::default(),
            //component_id_index_path: None,
            //log_destination: LogDestination::Syslog,
            //log_all_events: false,
            builtin_boot_resolver: BuiltinBootResolver::None,
            //realm_builder_resolver_and_runner: RealmBuilderResolverAndRunner::None,
            //abi_revision_policy: Default::default(),
            //vmex_source: Default::default(),
        }
    }
}
