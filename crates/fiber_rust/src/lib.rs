// Copyright 2022 MeshX Contributors. All rights reserved.

use fiber_sys as sys;

use crate::handle::{Handle, HandleRef, AsHandleRef, HandleBased};
use crate::info::ObjectQuery;
use crate::status::Status;

mod handle;
mod info;
mod process;
mod rights;
mod status;

// Implements the HandleBased traits for a Handle newtype struct
#[macro_export]
macro_rules! impl_handle_based {
    ($type_name:path) => {
        impl AsHandleRef for $type_name {
            fn as_handle_ref(&self) -> HandleRef<'_> {
                self.0.as_handle_ref()
            }
        }
        impl From<Handle> for $type_name {
            fn from(handle: Handle) -> Self {
                $type_name(handle)
            }
        }
        impl From<$type_name> for Handle {
            fn from(x: $type_name) -> Handle {
                x.0
            }
        }
        impl HandleBased for $type_name {}
    };
}

// Creates associated constants of TypeName of the form
// `pub const NAME: TypeName = TypeName(path::to::value);`
// and provides a private `assoc_const_name` method and a `Debug` implementation
// for the type based on `$name`.
// If multiple names match, the first will be used in `name` and `Debug`.
#[macro_export]
macro_rules! assoc_values {
    ($typename:ident, [$($(#[$attr:meta])* $name:ident = $value:path;)*]) => {
        #[allow(non_upper_case_globals)]
        impl $typename {
            $(
                $(#[$attr])*
                pub const $name: $typename = $typename($value);
            )*
            fn assoc_const_name(&self) -> Option<&'static str> {
                match self.0 {
                    $(
                        $(#[$attr])*
                        $value => Some(stringify!($name)),
                    )*
                    _ => None,
                }
            }
        }
        impl ::std::fmt::Debug for $typename {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str(concat!(stringify!($typename), "("))?;
                match self.assoc_const_name() {
                    Some(name) => f.write_str(&name)?,
                    None => ::std::fmt::Debug::fmt(&self.0, f)?,
                }
                f.write_str(")")
            }
        }
    }
}

/// Convenience re-export of `Status::ok`.
pub fn ok(raw: sys::fx_status_t) -> Result<(), Status> {
    Status::ok(raw)
}

/// Query information about a zircon object.
/// Returns `(num_returned, num_remaining)` on success.
pub fn object_get_info<Q: ObjectQuery>(handle: HandleRef<'_>, out: &mut [Q::InfoTy]) -> Result<(), Status> {
    let status = unsafe {
        sys::fx_object_get_info(
            handle.raw_handle(),
            *Q::TOPIC,
            out.as_mut_ptr() as *mut u8,
            std::mem::size_of_val(out),
        )
    };
    ok(status)
}
