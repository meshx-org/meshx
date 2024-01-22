// Copyright 2019 The Fuchsia Authors. All rights reserved.
// Copyright 2024 MeshX Contributors. All rights reserved.

//! The `Service` trait and its trait-object wrappers.

use fiber_rust as fx;

/// `Service` connects channels to service instances.
///
/// Note that this trait is implemented by the `FidlService` type.
pub trait Service {
    /// The type of the value yielded by the `spawn_service` callback.
    type Output;
    /// Create a new instance of the service on the provided `zx::Channel`.
    ///
    /// The value returned by this function will be yielded from the stream
    /// output of the `ServiceFs` type.
    fn connect(&mut self, channel: fx::Channel) -> Option<Self::Output>;
}

/// A `!Send` (non-thread-safe) trait object encapsulating a `Service` with
/// the given `Output` type.
///
/// Types which implement the `Service` trait can be converted to objects of
/// this type via the `From`/`Into` traits.
pub struct ServiceObjLocal<'a, Output>(Box<dyn Service<Output = Output> + 'a>);

/// A trait implemented by both `ServiceObj` and `ServiceObjLocal` that
/// allows code to be generic over thread-safety.
///
/// Code that uses `ServiceObj` will require `Send` bounds but will be
/// multithreaded-capable, while code that uses `ServiceObjLocal` will
/// allow non-`Send` types but will be restricted to singlethreaded
/// executors.
pub trait ServiceObjTrait {
    /// The output type of the underlying `Service`.
    type Output;
    /// Get a mutable reference to the underlying `Service` trait object.
    fn service(&mut self) -> &mut dyn Service<Output = Self::Output>;
}

impl<'a, Output> ServiceObjTrait for ServiceObjLocal<'a, Output> {
    type Output = Output;
    fn service(&mut self) -> &mut dyn Service<Output = Self::Output> {
        &mut *self.0
    }
}