// Copyright 2022 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use thiserror::Error;
use url::Url;

use crate::clonable_error::ClonableError;

/// The response returned from a Resolver. This struct is derived from the FIDL
/// [`fuchsia.component.resolution.Component`][fidl_fuchsia_component_resolution::Component]
/// table, except that the opaque binary ComponentDecl has been deserialized and validated.
#[derive(Debug)]
pub struct ResolvedComponent {
    /// The url used to resolve this component.
    pub resolved_url: String,
    // The package context, from the component resolution context returned by
    // the resolver.
    //pub context_to_resolve_children: Option<ComponentResolutionContext>,
    //pub decl: cm_rust::ComponentDecl,
    pub package: Option<ResolvedPackage>,
    //pub config_values: Option<cm_rust::ConfigValuesData>,
    //pub abi_revision: Option<AbiRevision>,
}

/// The response returned from a Resolver. This struct is derived from the FIDL
/// [`fuchsia.component.resolution.Package`][fidl_fuchsia_component_resolution::Package]
/// table.
#[derive(Debug)]
pub struct ResolvedPackage {
    /// The package url.
    pub url: String,
    // The package directory client proxy.
    //pub directory: fidl::endpoints::ClientEnd<fio::DirectoryMarker>,
}

/// Errors produced by built-in `Resolver`s and `resolving` APIs.
#[derive(Debug, Error, Clone)]
pub enum ResolverError {
    #[error("an unexpected error occurred: {0}")]
    Internal(#[source] ClonableError),
    #[error("package not found: {0}")]
    PackageNotFound(#[source] ClonableError),
}

/// Indicates the kind of `ComponentAddress`, and holds `ComponentAddress`
/// properties specific to its kind. Note that there is no kind for a relative
/// resource component URL (a URL that only contains a resource fragment, such
/// as `#meta/comp.cm`) because `ComponentAddress::from()` will translate a
/// resource fragment component URL into one of the fully-resolvable
/// `ComponentAddress`s.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentAddress {
    /// A fully-qualified component URL.
    Absolute { url: Url },


    /// A relative Component URL, starting with the package path; for example a
    /// subpackage relative URL such as "needed_package#meta/dep_component.cm".
    RelativePath {
        /// This is the scheme of the ancestor component's absolute URL, used to identify the
        /// `Resolver` in a `ResolverRegistry`.
        scheme: String,

        /// The relative URL, represented as a `url::Url` with the `relative:///` base prefix.
        /// `url::Url` cannot represent relative urls directly.
        url: Url,

        // An opaque value (from the perspective of component resolution)
        // required by the resolver when resolving a relative package path.
        // For a given child component, this property is populated from a
        // parent component's `resolution_context`, as returned by the parent
        // component's resolver.
        //context: ComponentResolutionContext,
    },
}
