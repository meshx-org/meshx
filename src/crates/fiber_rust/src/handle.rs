// Copyright 2023 MeshX Contributors. All rights reserved.
// Copyright 2018 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! Type-safe bindings for Fiber handles.

use fiber_status::Status;
use fiber_sys as sys;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;

use crate::info::{ObjectQuery, Topic};
use crate::rights::Rights;
use crate::{assoc_values, object_get_info, ok, Port, Signals, Time, WaitAsyncOpts};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct Koid(sys::fx_koid_t);

impl Koid {
    pub fn from_raw(raw: sys::fx_koid_t) -> Koid {
        Koid(raw)
    }
    pub fn raw_koid(&self) -> sys::fx_koid_t {
        self.0
    }
}

/// An object representing a Zircon
/// [handle](https://fuchsia.dev/fuchsia-src/concepts/objects/handles).
///
/// Internally, it is represented as a 32-bit integer, but this wrapper enforces
/// strict ownership semantics. The `Drop` implementation closes the handle.
///
/// This type represents the most general reference to a kernel object, and can
/// be interconverted to and from more specific types. Those conversions are not
/// enforced in the type system; attempting to use them will result in errors
/// returned by the kernel. These conversions don't change the underlying
/// representation, but do change the type and thus what operations are available.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct Handle(sys::fx_handle_t);

impl AsHandleRef for Handle {
    fn as_handle_ref(&self) -> HandleRef<'_> {
        Unowned {
            inner: ManuallyDrop::new(Handle(self.0)),
            marker: PhantomData,
        }
    }
}

impl HandleBased for Handle {}

impl Drop for Handle {
    fn drop(&mut self) {
        if self.0 != sys::FX_HANDLE_INVALID {
            unsafe { sys::fx_handle_close(self.0) };
        }
    }
}

impl Handle {
    /// Initialize a handle backed by ZX_HANDLE_INVALID, the only safe non-handle.
    #[inline(always)]
    pub const fn invalid() -> Handle {
        Handle(sys::FX_HANDLE_INVALID)
    }

    #[allow(clippy::missing_safety_doc)] // TODO(fxbug.dev/99066)
    /// If a raw handle is obtained from some other source, this method converts
    /// it into a type-safe owned handle.
    pub const unsafe fn from_raw(raw: sys::fx_handle_t) -> Handle {
        Handle(raw)
    }

    pub fn is_invalid(&self) -> bool {
        self.0 == sys::FX_HANDLE_INVALID
    }

    pub fn replace(self, rights: Rights) -> Result<Handle, Status> {
        let handle = self.0;
        let mut out = 0;
        let status = unsafe { sys::fx_handle_replace(handle, rights.bits(), &mut out) };
        Status::ok(status).map(|()| Handle(out))
    }
}

/// A borrowed value of type `T`.
///
/// This is primarily used for working with borrowed values of `HandleBased` types.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct Unowned<'a, T> {
    inner: ManuallyDrop<T>,
    marker: PhantomData<&'a T>,
}

impl<'a, T> ::std::ops::Deref for Unowned<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}

pub type HandleRef<'a> = Unowned<'a, Handle>;

impl<'a, T: HandleBased> Unowned<'a, T> {
    #[allow(clippy::missing_safety_doc)] // TODO(fxbug.dev/99066)
    /// Create a `HandleRef` from a raw handle. Use this method when you are given a raw handle but
    /// should not take ownership of it. Examples include process-global handles like the root
    /// VMAR. This method should be called with an explicitly provided lifetime that must not
    /// outlive the lifetime during which the handle is owned by the current process. It is unsafe
    /// because most of the time, it is better to use a `Handle` to prevent leaking resources.
    pub unsafe fn from_raw_handle(handle: sys::fx_handle_t) -> Self {
        Unowned {
            inner: ManuallyDrop::new(T::from(Handle::from_raw(handle))),
            marker: PhantomData,
        }
    }

    pub fn raw_handle(&self) -> sys::fx_handle_t {
        self.inner.raw_handle()
    }

    pub fn duplicate(&self, rights: Rights) -> Result<T, Status> {
        let mut out = 0;
        let status = unsafe { sys::fx_handle_duplicate(self.raw_handle(), rights.bits(), &mut out) };
        ok(status).map(|()| T::from(Handle(out)))
    }

    pub fn signal(&self, clear_mask: Signals, set_mask: Signals) -> Result<(), Status> {
        let status = unsafe { sys::fx_object_signal(self.raw_handle(), clear_mask.bits(), set_mask.bits()) };
        ok(status)
    }

    pub fn wait(&self, signals: Signals, deadline: Time) -> Result<Signals, Status> {
        let mut pending = Signals::empty().bits();
        let status =
            unsafe { sys::fx_object_wait_one(self.raw_handle(), signals.bits(), deadline.into_nanos(), &mut pending) };
        ok(status).map(|()| Signals::from_bits_truncate(pending))
    }

    pub fn wait_async(&self, port: &Port, key: u64, signals: Signals, options: WaitAsyncOpts) -> Result<(), Status> {
        let status = unsafe {
            sys::fx_object_wait_async(
                self.raw_handle(),
                port.raw_handle(),
                key,
                signals.bits(),
                options.bits(),
            )
        };
        ok(status)
    }
}

/// A trait to get a reference to the underlying handle of an object.
pub trait AsHandleRef {
    /// Get a reference to the handle. One important use of such a reference is
    /// for `object_wait_many`.
    fn as_handle_ref(&self) -> HandleRef<'_>;

    /// Interpret the reference as a raw handle (an integer type). Two distinct
    /// handles will have different raw values (so it can perhaps be used as a
    /// key in a data structure).
    fn raw_handle(&self) -> sys::fx_handle_t {
        self.as_handle_ref().inner.0
    }

    /// Set and clear userspace-accessible signal bits on an object. Wraps the
    /// [zx_object_signal](https://fuchsia.dev/fuchsia-src/reference/syscalls/object_signal.md)
    /// syscall.
    fn signal_handle(&self, clear_mask: Signals, set_mask: Signals) -> Result<(), Status> {
        self.as_handle_ref().signal(clear_mask, set_mask)
    }

    /// Waits on a handle. Wraps the
    /// [zx_object_wait_one](https://fuchsia.dev/fuchsia-src/reference/syscalls/object_wait_one.md)
    /// syscall.
    fn wait_handle(&self, signals: Signals, deadline: Time) -> Result<Signals, Status> {
        self.as_handle_ref().wait(signals, deadline)
    }

    /// Causes packet delivery on the given port when the object changes state and matches signals.
    /// [zx_object_wait_async](https://fuchsia.dev/fuchsia-src/reference/syscalls/object_wait_async.md)
    /// syscall.
    fn wait_async_handle(&self, port: &Port, key: u64, signals: Signals, options: WaitAsyncOpts) -> Result<(), Status> {
        self.as_handle_ref().wait_async(port, key, signals, options)
    }

    /// Wraps the
    /// [zx_object_get_info](https://fuchsia.dev/fuchsia-src/reference/syscalls/object_get_info.md)
    /// syscall for the ZX_INFO_HANDLE_BASIC topic.
    fn basic_info(&self) -> Result<HandleBasicInfo, Status> {
        let mut info = sys::fx_info_handle_basic_t::default();
        object_get_info::<HandleBasicInfoQuery>(self.as_handle_ref(), std::slice::from_mut(&mut info))
            .map(|_| HandleBasicInfo::from(info))
    }

    /// Returns the koid (kernel object ID) for this handle.
    fn get_koid(&self) -> Result<Koid, Status> {
        self.basic_info().map(|info| info.koid)
    }
}

impl<'a, T: HandleBased> AsHandleRef for Unowned<'a, T> {
    fn as_handle_ref(&self) -> HandleRef<'_> {
        Unowned {
            inner: ManuallyDrop::new(Handle(self.raw_handle())),
            marker: PhantomData,
        }
    }
}

/// A trait implemented by all handle-based types.
///
/// Note: it is reasonable for user-defined objects wrapping a handle to implement
/// this trait. For example, a specific interface in some protocol might be
/// represented as a newtype of `Channel`, and implement the `as_handle_ref`
/// method and the `From<Handle>` trait to facilitate conversion from and to the
/// interface.
pub trait HandleBased: AsHandleRef + From<Handle> + Into<Handle> {
    /// Duplicate a handle, possibly reducing the rights available. Wraps the
    /// [fx_handle_duplicate](https://fuchsia.dev/fuchsia-src/reference/syscalls/handle_duplicate.md)
    /// syscall.
    fn duplicate_handle(&self, rights: Rights) -> Result<Self, Status> {
        self.as_handle_ref().duplicate(rights).map(|handle| Self::from(handle))
    }

    /// Create a replacement for a handle, possibly reducing the rights available. This invalidates
    /// the original handle. Wraps the
    /// [fx_handle_replace](https://fuchsia.dev/fuchsia-src/reference/syscalls/handle_replace.md)
    /// syscall.
    fn replace_handle(self, rights: Rights) -> Result<Self, Status> {
        <Self as Into<Handle>>::into(self)
            .replace(rights)
            .map(|handle| Self::from(handle))
    }

    /// Converts the value into its inner handle.
    ///
    /// This is a convenience function which simply forwards to the `Into` trait.
    fn into_handle(self) -> Handle {
        self.into()
    }

    /// Converts the handle into it's raw representation.
    ///
    /// The caller takes ownership over the raw handle, and must close or transfer it to avoid a handle leak.
    fn into_raw(self) -> sys::fx_handle_t {
        let h = self.into_handle();
        let r = h.0;
        std::mem::forget(h);
        r
    }

    /// Creates an instance of this type from a handle.
    ///
    /// This is a convenience function which simply forwards to the `From` trait.
    fn from_handle(handle: Handle) -> Self {
        Self::from(handle)
    }

    /// Creates an instance of another handle-based type from this value's inner handle.
    fn into_handle_based<H: HandleBased>(self) -> H {
        H::from_handle(self.into_handle())
    }

    /// Creates an instance of this type from the inner handle of another
    /// handle-based type.
    fn from_handle_based<H: HandleBased>(h: H) -> Self {
        Self::from_handle(h.into_handle())
    }

    fn is_invalid_handle(&self) -> bool {
        self.as_handle_ref().is_invalid()
    }
}

/// A trait implemented by all handles for objects which have a peer.
pub trait Peered: HandleBased {
    /// Set and clear userspace-accessible signal bits on the object's peer. Wraps the
    /// [zx_object_signal_peer][osp] syscall.
    ///
    /// [osp]: https://fuchsia.dev/fuchsia-src/reference/syscalls/object_signal_peer.md
    fn signal_peer(&self, clear_mask: Signals, set_mask: Signals) -> Result<(), Status> {
        let handle = self.raw_handle();
        let status = unsafe { sys::fx_object_signal_peer(handle, clear_mask.bits(), set_mask.bits()) };
        ok(status)
    }
}

/// Fiber object types.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct ObjectType(sys::fx_obj_type_t);

assoc_values!(ObjectType, [
    NONE            = sys::FX_OBJ_TYPE_NONE;
    PROCESS         = sys::FX_OBJ_TYPE_PROCESS;
    CHANNEL         = sys::FX_OBJ_TYPE_CHANNEL;
    PORT            = sys::FX_OBJ_TYPE_PORT;
    JOB             = sys::FX_OBJ_TYPE_JOB;
    VMO             = sys::FX_OBJ_TYPE_VMO;
]);

impl ObjectType {
    /// Creates an `ObjectType` from the underlying zircon type.
    pub const fn from_raw(raw: sys::fx_obj_type_t) -> Self {
        Self(raw)
    }

    /// Converts `ObjectType` into the underlying zircon type.
    pub const fn into_raw(self) -> sys::fx_obj_type_t {
        self.0
    }
}

/// Basic information about a handle.
///
/// Wrapper for data returned from [Handle::basic_info()].
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct HandleBasicInfo {
    pub koid: Koid,
    pub rights: Rights,
    pub object_type: ObjectType,
    pub related_koid: Koid,
    pub reserved: u32,
}

impl Default for HandleBasicInfo {
    fn default() -> Self {
        Self::from(sys::fx_info_handle_basic_t::default())
    }
}

impl From<sys::fx_info_handle_basic_t> for HandleBasicInfo {
    fn from(info: sys::fx_info_handle_basic_t) -> Self {
        let sys::fx_info_handle_basic_t {
            koid,
            rights,
            type_,
            related_koid,
            reserved,
        } = info;

        // Note lossy conversion of Rights and HandleProperty here if either of those types are out
        // of date or incomplete.
        HandleBasicInfo {
            koid: Koid(koid),
            rights: Rights::from_bits_truncate(rights),
            object_type: ObjectType(type_),
            related_koid: Koid(related_koid),
            reserved,
        }
    }
}

// fx_info_handle_basic_t is able to be safely replaced with a byte representation and is a PoD
// type.
struct HandleBasicInfoQuery;

unsafe impl ObjectQuery for HandleBasicInfoQuery {
    const TOPIC: Topic = Topic::HANDLE_BASIC;
    type InfoTy = sys::fx_info_handle_basic_t;
}

/// Handle operation.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum HandleOp<'a> {
    Move(Handle),
    Duplicate(HandleRef<'a>),
}

/// Operation to perform on handles during write.
/// Based on fx_handle_disposition_t, but does not match the same layout.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct HandleDisposition<'a> {
    pub handle_op: HandleOp<'a>,
    pub object_type: ObjectType,
    pub rights: Rights,
    pub result: Status,
}

impl HandleDisposition<'_> {
    pub fn into_raw<'a>(self) -> sys::fx_handle_disposition_t {
        match self.handle_op {
            HandleOp::Move(mut handle) => sys::fx_handle_disposition_t {
                operation: sys::FX_HANDLE_OP_MOVE,
                handle: std::mem::replace(&mut handle, Handle::invalid()).into_raw(),
                type_: self.object_type.0,
                rights: self.rights.bits(),
                result: self.result.into_raw(),
            },
            HandleOp::Duplicate(handle_ref) => sys::fx_handle_disposition_t {
                operation: sys::FX_HANDLE_OP_DUPLICATE,
                handle: handle_ref.raw_handle(),
                type_: self.object_type.0,
                rights: self.rights.bits(),
                result: self.result.into_raw(),
            },
        }
    }
}

/// Information on handles that were read.
/// Based on zx_handle_info_t.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(C)]
pub struct HandleInfo {
    pub handle: Handle,
    pub object_type: ObjectType,
    pub rights: Rights,
}

impl HandleInfo {
    /// # Safety
    ///
    /// See [`Handle::from_raw`] for requirements about the validity and closing
    /// of `raw.handle`.
    ///
    /// `raw.rights` must be a bitwise combination of one or more [`Rights`]
    /// with no additional bits set.
    ///
    /// Note that while `raw.ty` _should_ correspond to the type of the handle,
    /// that this is not required for safety.
    pub const unsafe fn from_raw(raw: sys::fx_handle_info_t) -> HandleInfo {
        HandleInfo {
            handle: Handle::from_raw(raw.handle),
            object_type: ObjectType(raw.ty),
            rights: Rights::from_bits_unchecked(raw.rights),
        }
    }
}
