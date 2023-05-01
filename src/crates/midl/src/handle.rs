// Copyright 2023 MeshX Contributors. All rights reserved.
// Copyright 2019 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! A portable representation of handle-like objects for fidl.

pub use meshx_handles::*;

pub use meshx_async::Channel as AsyncChannel;
pub use meshx_async::OnSignals;
// pub use meshx_async::Socket as AsyncSocket;

/// MeshX implementation of handles just aliases the fiber library
pub mod meshx_handles {
    use fiber_rust as fx;

    pub use fx::AsHandleRef;
    pub use fx::Handle;
    pub use fx::HandleBased;
    pub use fx::HandleDisposition;
    pub use fx::HandleInfo;
    pub use fx::HandleOp;
    pub use fx::HandleRef;
    pub use fx::MessageBufEtc;
    pub use fx::ObjectType;
    pub use fx::Peered;
    pub use fx::Rights;
    pub use fx::Signals;
    // pub use fx::SocketOpts;
    pub use fx::Status;

    pub use meshx_async::invoke_for_handle_types;

    macro_rules! meshx_handle {
        ($x:tt, $docname:expr, $name:ident, $value:expr, Stub) => {
            /// Stub implementation of Zircon handle type $x.
            #[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
            #[repr(transparent)]
            pub struct $x(fx::Handle);

            impl fx::AsHandleRef for $x {
                fn as_handle_ref(&self) -> HandleRef<'_> {
                    self.0.as_handle_ref()
                }
            }
            impl From<Handle> for $x {
                fn from(handle: Handle) -> Self {
                    $x(handle)
                }
            }
            impl From<$x> for Handle {
                fn from(x: $x) -> Handle {
                    x.0
                }
            }
            impl fx::HandleBased for $x {}
        };
        ($x:tt, $docnamee:expr, $name:ident, $value:expr, $availability:tt) => {
            pub use fx::$x;
        };
    }

    invoke_for_handle_types!(meshx_handle);
}
