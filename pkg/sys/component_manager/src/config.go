package component_manager

/// Runtime configuration options.
/// This configuration intended to be "global", in that the same configuration
/// is applied throughout a given running instance of component_manager.

type RuntimeConfig  struct{
    /// How many children, maximum, are returned by a call to `ChildIterator.next()`.
    list_children_batch_size uint16

    /// Security policy configuration.
    securityPolicy SecurityPolicy

    /// If true, component manager will be in debug mode. In this mode, component manager
    /// provides the `EventSource` protocol and exposes this protocol. The root component
    /// must be manually started using the LifecycleController protocol in the hub.
    ///
    /// This is done so that an external component (say an integration test) can subscribe
    /// to events before the root component has started.
    debug bool

    // If true, component_manager will serve an instance of fuchsia.process.Launcher and use this
    // launcher for the built-in ELF component runner. The root component can additionally
    // use and/or offer this service using '/builtin/fuchsia.process.Launcher' from realm.
    // This flag exists because the built-in process launcher *only* works when
    // component_manager runs under a job that has ZX_POL_NEW_PROCESS set to allow, like the root
    // job. Otherwise, the component_manager process cannot directly create process through
    // zx_process_create. When we run component_manager elsewhere, like in test environments, it
    // has to use the fuchsia.process.Launcher service provided through its namespace instead.
    use_builtin_process_launcher bool

    // The list of capabilities offered from component manager's namespace.
    namespace_capabilities []cm_rust.CapabilityDecl

    // The list of capabilities offered from component manager as built-in capabilities.
    builtin_capabilities []cm_rust.CapabilityDecl

    // Which builtin resolver to use for the fuchsia-pkg scheme. If not supplied this defaults to
    // the NONE option.
    builtin_pkg_resolver BuiltinPkgResolver

    // Determine what content to expose through the component manager's
    // outgoing directory.
    out_dir_contents OutDirContents

    // URL of the root component to launch. This field is used if no URL
    // is passed to component manager. If value is passed in both places, then
    // an error is raised.
    root_component_url Option<Url>

    // Path to the component ID index, parsed from
    // `fuchsia.component.internal.RuntimeConfig.component_id_index_path`.
    component_id_index_path Option<String>

    // Where to log to.
    log_destination LogDestination

    // If true, component manager will log all events dispatched in the topology.
    pub log_all_events: bool,

    // Which builtin resolver to use for the fuchsia-boot scheme. If not supplied this defaults to
    // the NONE option.
    builtin_boot_resolver: BuiltinBootResolver,

    // If true, allow components to set the `OnTerminate=REBOOT` option.
    //
    // This lets a parent component designate that the system should reboot if a child terminates
    // (except when it's shut down).
    reboot_on_terminate_enabled: bool,

    // If and how the realm builder resolver and runner are enabled.
     realm_builder_resolver_and_runner: RealmBuilderResolverAndRunner,
}