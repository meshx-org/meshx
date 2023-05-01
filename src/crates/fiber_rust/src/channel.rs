// Copyright 2023 MeshX Contributors. All rights reserved.
// Copyright 2017 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! Type-safe bindings for Zircon channel objects.

use crate::{impl_handle_based, ok};
use crate::{
    size_to_u32_sat, usize_into_u32, AsHandleRef, Handle, HandleBased, HandleDisposition, HandleInfo, HandleOp,
    HandleRef, ObjectType, Peered, Rights, Status, Time,
};
use fiber_sys as sys;
use std::mem::{self, MaybeUninit};

impl HandleDisposition<'_> {
    const fn invalid<'a>() -> HandleDisposition<'a> {
        HandleDisposition {
            handle_op: HandleOp::Move(Handle::invalid()),
            object_type: ObjectType::NONE,
            rights: Rights::NONE,
            result: Status::OK,
        }
    }
}

/// An object representing a Zircon
/// [channel](https://fuchsia.dev/fuchsia-src/concepts/objects/channel.md).
///
/// As essentially a subtype of `Handle`, it can be freely interconverted.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct Channel(Handle);
impl_handle_based!(Channel);
impl Peered for Channel {}

impl Channel {
    /// Create a channel, resulting in a pair of `Channel` objects representing both
    /// sides of the channel. Messages written into one may be read from the opposite.
    ///
    /// Wraps the
    /// [fx_channel_create](https://fuchsia.dev/fuchsia-src/reference/syscalls/channel_create.md)
    /// syscall.
    ///
    /// # Panics
    ///
    /// If the process' job policy denies channel creation or the kernel reports no memory
    /// available to create a new channel.
    pub fn create() -> (Self, Self) {
        unsafe {
            let mut handle0 = 0;
            let mut handle1 = 0;
            let opts = 0;
            ok(sys::fx_channel_create(opts, &mut handle0, &mut handle1))
                .expect("channel creation always succeeds except with OOM or when job policy denies it");
            (Self(Handle::from_raw(handle0)), Self(Handle::from_raw(handle1)))
        }
    }

    /// Read a message from a channel. Wraps the
    /// [fx_channel_read](https://fuchsia.dev/fuchsia-src/reference/syscalls/channel_read.md)
    /// syscall.
    ///
    /// If the slice lacks the capacity to hold the pending message,
    /// returns an `Err` with the number of bytes and number of handles needed.
    /// Otherwise returns an `Ok` with the result as usual.
    /// If both the outer and inner `Result`s are `Ok`, then the caller can
    /// assume that the `handles` array is initialized.
    ///
    /// Note that `read_slice` may call `read_raw` with some uninitialized
    /// elements because it resizes the input vector to its capacity
    /// without initializing all of the elements.
    pub fn read_raw(
        &self,
        bytes: &mut [u8],
        handles: &mut [MaybeUninit<Handle>],
    ) -> Result<(Result<(), Status>, usize, usize), (usize, usize)> {
        unsafe {
            let raw_handle = self.raw_handle();
            let mut actual_bytes = 0;
            let mut actual_handles = 0;
            let status = ok(sys::fx_channel_read(
                raw_handle,
                0,
                bytes.as_mut_ptr(),
                handles.as_mut_ptr() as *mut _,
                bytes.len() as u32,
                handles.len() as u32,
                &mut actual_bytes,
                &mut actual_handles,
            ));
            if status == Err(Status::BUFFER_TOO_SMALL) {
                Err((actual_bytes as usize, actual_handles as usize))
            } else {
                Ok((status, actual_bytes as usize, actual_handles as usize))
            }
        }
    }

    /// Read a message from a channel.
    ///
    /// Note that this method can cause internal reallocations in the `MessageBuf`
    /// if it is lacks capacity to hold the full message. If such reallocations
    /// are not desirable, use `read_raw` instead.
    pub fn read(&self, buf: &mut MessageBuf) -> Result<(), Status> {
        let (bytes, handles) = buf.split_mut();
        self.read_split(bytes, handles)
    }

    /// Read a message from a channel into a separate byte vector and handle vector.
    ///
    /// Note that this method can cause internal reallocations in the `Vec`s
    /// if they lacks capacity to hold the full message. If such reallocations
    /// are not desirable, use `read_raw` instead.
    pub fn read_split(&self, bytes: &mut Vec<u8>, handles: &mut Vec<Handle>) -> Result<(), Status> {
        loop {
            unsafe {
                bytes.set_len(bytes.capacity());
                handles.set_len(handles.capacity());
            }

            let handle_slice: &mut [Handle] = handles;
            match self.read_raw(bytes, unsafe { mem::transmute(handle_slice) }) {
                Ok((result, num_bytes, num_handles)) => {
                    unsafe {
                        bytes.set_len(num_bytes);
                        handles.set_len(num_handles);
                    }
                    return result;
                }
                Err((num_bytes, num_handles)) => {
                    ensure_capacity(bytes, num_bytes);
                    ensure_capacity(handles, num_handles);
                }
            }
        }
    }

    /// Read a message from a channel.
    /// Wraps the [fx_channel_read_etc](https://fuchsia.dev/fuchsia-src/reference/syscalls/channel_read_etc.md)
    /// syscall.
    ///
    /// This differs from `read_raw` in that it returns extended information on
    /// the handles.
    ///
    /// If the slice lacks the capacity to hold the pending message,
    /// returns an `Err` with the number of bytes and number of handles needed.
    /// Otherwise returns an `Ok` with the result as usual.
    /// If both the outer and inner `Result`s are `Ok`, then the caller can
    /// assume that the `handle_infos` array is initialized.
    ///
    /// Note that `read_etc_slice` may call `read_etc_raw` with some
    /// uninitialized elements because it resizes the input vector to its
    /// capacity without initializing all of the elements.
    pub fn read_etc_raw(
        &self,
        bytes: &mut [u8],
        handle_infos: &mut [MaybeUninit<HandleInfo>],
    ) -> Result<(Result<(), Status>, usize, usize), (usize, usize)> {
        unsafe {
            let raw_handle = self.raw_handle();
            let mut fx_handle_infos: [MaybeUninit<sys::fx_handle_info_t>; sys::FX_CHANNEL_MAX_MSG_HANDLES as usize] =
                MaybeUninit::uninit().assume_init();
            let mut actual_bytes = 0;
            let mut actual_handle_infos = 0;

            let status = ok(sys::fx_channel_read_etc(
                raw_handle,
                0,
                bytes.as_mut_ptr(),
                fx_handle_infos.as_mut_ptr() as *mut sys::fx_handle_info_t,
                bytes.len() as u32,
                handle_infos.len() as u32,
                &mut actual_bytes,
                &mut actual_handle_infos,
            ));

            if status == Err(Status::BUFFER_TOO_SMALL) {
                Err((actual_bytes as usize, actual_handle_infos as usize))
            } else {
                Ok((
                    status.map(|()| {
                        for i in 0..actual_handle_infos as usize {
                            std::mem::swap(
                                &mut handle_infos[i],
                                &mut MaybeUninit::new(HandleInfo::from_raw(fx_handle_infos[i].assume_init())),
                            );
                        }
                    }),
                    actual_bytes as usize,
                    actual_handle_infos as usize,
                ))
            }
        }
    }

    /// Read a message from a channel.
    ///
    /// This differs from `read` in that it returns extended information on
    /// the handles.
    ///
    /// Note that this method can cause internal reallocations in the `MessageBufEtc`
    /// if it is lacks capacity to hold the full message. If such reallocations
    /// are not desirable, use `read_etc_raw` instead.
    pub fn read_etc(&self, buf: &mut MessageBufEtc) -> Result<(), Status> {
        let (bytes, handles) = buf.split_mut();
        self.read_etc_split(bytes, handles)
    }

    /// Read a message from a channel into a separate byte vector and handle vector.
    ///
    /// This differs from `read_split` in that it returns extended information on
    /// the handles.
    ///
    /// Note that this method can cause internal reallocations in the `Vec`s
    /// if they lacks capacity to hold the full message. If such reallocations
    /// are not desirable, use `read_raw` instead.
    pub fn read_etc_split(&self, bytes: &mut Vec<u8>, handle_infos: &mut Vec<HandleInfo>) -> Result<(), Status> {
        loop {
            unsafe {
                bytes.set_len(bytes.capacity());
                handle_infos.set_len(handle_infos.capacity());
            }
            let handle_info_slice: &mut [HandleInfo] = handle_infos;
            match self.read_etc_raw(bytes, unsafe { std::mem::transmute(handle_info_slice) }) {
                Ok((result, num_bytes, num_handle_infos)) => {
                    unsafe {
                        bytes.set_len(num_bytes);
                        handle_infos.set_len(num_handle_infos);
                    }
                    return result;
                }
                Err((num_bytes, num_handle_infos)) => {
                    ensure_capacity(bytes, num_bytes);
                    ensure_capacity(handle_infos, num_handle_infos);
                }
            }
        }
    }

    /// Write a message to a channel. Wraps the
    /// [zx_channel_write](https://fuchsia.dev/fuchsia-src/reference/syscalls/channel_write.md)
    /// syscall.
    pub fn write(&self, bytes: &[u8], handles: &mut [Handle]) -> Result<(), Status> {
        let n_bytes = usize_into_u32(bytes.len()).map_err(|_| Status::OUT_OF_RANGE)?;
        let n_handles = usize_into_u32(handles.len()).map_err(|_| Status::OUT_OF_RANGE)?;
        unsafe {
            let status = sys::fx_channel_write(
                self.raw_handle(),
                0,
                bytes.as_ptr(),
                n_bytes,
                handles.as_ptr() as *const sys::fx_handle_t,
                n_handles,
            );
            // Handles are consumed by zx_channel_write so prevent the destructor from being called.
            for handle in handles {
                std::mem::forget(std::mem::replace(handle, Handle::invalid()));
            }
            ok(status)?;
            Ok(())
        }
    }

    /// Write a message to a channel. Wraps the
    /// [zx_channel_write_etc](https://fuchsia.dev/fuchsia-src/reference/syscalls/channel_write_etc.md)
    /// syscall.
    pub fn write_etc(&self, bytes: &[u8], handle_dispositions: &mut [HandleDisposition<'_>]) -> Result<(), Status> {
        let n_bytes = usize_into_u32(bytes.len()).map_err(|_| Status::OUT_OF_RANGE)?;
        let n_handle_dispositions = usize_into_u32(handle_dispositions.len()).map_err(|_| Status::OUT_OF_RANGE)?;
        if n_handle_dispositions > sys::FX_CHANNEL_MAX_MSG_HANDLES {
            // don't let the kernel check this bound for us because we have a fixed size array below
            return Err(Status::OUT_OF_RANGE);
        }

        unsafe {
            let mut zx_handle_dispositions: [std::mem::MaybeUninit<sys::fx_handle_disposition_t>;
                sys::FX_CHANNEL_MAX_MSG_HANDLES as usize] = std::mem::MaybeUninit::uninit().assume_init();
            for i in 0..n_handle_dispositions as usize {
                let handle_disposition = std::mem::replace(&mut handle_dispositions[i], HandleDisposition::invalid());
                zx_handle_dispositions[i] = std::mem::MaybeUninit::new(handle_disposition.into_raw());
            }
            let status = sys::fx_channel_write_etc(
                self.raw_handle(),
                0,
                bytes.as_ptr(),
                n_bytes,
                zx_handle_dispositions.as_mut_ptr() as *mut sys::fx_handle_disposition_t,
                n_handle_dispositions,
            );
            ok(status)?;
            Ok(())
        }
    }

    /// Send a message consisting of the given bytes and handles to a channel and await a reply.
    ///
    /// The first four bytes of the written and read back messages are treated as a transaction ID
    /// of type `zx_txid_t`. The kernel generates a txid for the written message, replacing that
    /// part of the message as read from userspace. In other words, the first four bytes of
    /// `bytes` will be ignored, and the first four bytes of the response will contain a
    /// kernel-generated txid.
    ///
    /// This differs from `call`, in that it uses extended handle info.
    ///
    /// Wraps the
    /// [zx_channel_call_etc](https://fuchsia.dev/fuchsia-src/reference/syscalls/channel_call_etc.md)
    /// syscall.
    ///
    /// Note that unlike [`read_etc`][read_etc], the caller must ensure that the MessageBufEtc
    /// has enough capacity for the bytes and handles which will be received, as replies which are
    /// too large are discarded.
    ///
    /// On failure returns the both the main and read status.
    ///
    /// [read_etc]: struct.Channel.html#method.read_etc
    pub fn call_etc(
        &self,
        timeout: Time,
        bytes: &[u8],
        handle_dispositions: &mut [HandleDisposition<'_>],
        buf: &mut MessageBufEtc,
    ) -> Result<(), Status> {
        let write_num_bytes = usize_into_u32(bytes.len()).map_err(|_| Status::OUT_OF_RANGE)?;
        let write_num_handle_dispositions =
            usize_into_u32(handle_dispositions.len()).map_err(|_| Status::OUT_OF_RANGE)?;
        if write_num_handle_dispositions > sys::FX_CHANNEL_MAX_MSG_HANDLES {
            // don't let the kernel check this bound for us because we have a fixed size array below
            return Err(Status::OUT_OF_RANGE);
        }

        let mut fx_handle_dispositions: [std::mem::MaybeUninit<sys::fx_handle_disposition_t>;
            sys::FX_CHANNEL_MAX_MSG_HANDLES as usize] =
            [std::mem::MaybeUninit::uninit(); sys::FX_CHANNEL_MAX_MSG_HANDLES as usize];

        for i in 0..write_num_handle_dispositions as usize {
            let handle_disposition = std::mem::replace(&mut handle_dispositions[i], HandleDisposition::invalid());
            fx_handle_dispositions[i].write(handle_disposition.into_raw());
        }

        buf.clear();
        let read_num_bytes: u32 = size_to_u32_sat(buf.bytes.capacity());
        let read_num_handle_infos: u32 = size_to_u32_sat(buf.handle_infos.capacity());
        let mut fx_handle_infos: [std::mem::MaybeUninit<sys::fx_handle_info_t>;
            sys::FX_CHANNEL_MAX_MSG_HANDLES as usize] = unsafe { std::mem::MaybeUninit::uninit().assume_init() };

        let mut args = sys::fx_channel_call_etc_args_t {
            wr_bytes: bytes.as_ptr(),
            wr_handles: fx_handle_dispositions.as_mut_ptr() as *mut sys::fx_handle_disposition_t,
            rd_bytes: buf.bytes.as_mut_ptr(),
            rd_handles: fx_handle_infos.as_mut_ptr() as *mut sys::fx_handle_info_t,
            wr_num_bytes: write_num_bytes,
            wr_num_handles: write_num_handle_dispositions,
            rd_num_bytes: read_num_bytes,
            rd_num_handles: read_num_handle_infos,
        };
        let mut actual_read_bytes: u32 = 0;
        let mut actual_read_handle_infos: u32 = 0;

        let status = unsafe {
            Status::from_raw(sys::fx_channel_call_etc(
                self.raw_handle(),
                0,
                timeout.into_nanos(),
                &mut args,
                &mut actual_read_bytes,
                &mut actual_read_handle_infos,
            ))
        };

        unsafe {
            buf.ensure_capacity_handle_infos(actual_read_handle_infos as usize);
            for i in 0..actual_read_handle_infos as usize {
                buf.handle_infos
                    .push(HandleInfo::from_raw(fx_handle_infos[i].assume_init()));
            }
            buf.bytes.set_len(actual_read_bytes as usize);
        }

        if Status::OK == status {
            Ok(())
        } else {
            Err(status)
        }
    }
}

#[test]
pub fn test_handle_repr() {
    assert_eq!(::std::mem::size_of::<sys::fx_handle_t>(), 4);
    assert_eq!(::std::mem::size_of::<Handle>(), 4);
    assert_eq!(
        ::std::mem::align_of::<sys::fx_handle_t>(),
        ::std::mem::align_of::<Handle>()
    );

    // This test asserts that repr(transparent) still works for Handle -> zx_handle_t
    let n: Vec<sys::fx_handle_t> = vec![0, 100, 2 << 32 - 1];
    let v: Vec<Handle> = n.iter().map(|h| unsafe { Handle::from_raw(*h) }).collect();

    for (handle, raw) in v.iter().zip(n.iter()) {
        unsafe {
            assert_eq!(
                *(handle as *const _ as *const [u8; 4]),
                *(raw as *const _ as *const [u8; 4])
            );
        }
    }

    for h in v.into_iter() {
        ::std::mem::forget(h);
    }
}

impl AsRef<Channel> for Channel {
    fn as_ref(&self) -> &Self {
        &self
    }
}

/// A buffer for _receiving_ messages from a channel.
///
/// A `MessageBuf` is essentially a byte buffer and a vector of
/// handles, but move semantics for "taking" handles requires special handling.
///
/// Note that for sending messages to a channel, the caller manages the buffers,
/// using a plain byte slice and `Vec<Handle>`.
#[derive(Debug, Default)]
pub struct MessageBuf {
    bytes: Vec<u8>,
    handles: Vec<Handle>,
}

impl MessageBuf {
    /// Create a new, empty, message buffer.
    pub fn new() -> Self {
        Default::default()
    }

    /// Create a new non-empty message buffer.
    pub fn new_with(v: Vec<u8>, h: Vec<Handle>) -> Self {
        Self { bytes: v, handles: h }
    }

    /// Splits apart the message buf into a vector of bytes and a vector of handles.
    pub fn split_mut(&mut self) -> (&mut Vec<u8>, &mut Vec<Handle>) {
        (&mut self.bytes, &mut self.handles)
    }

    /// Splits apart the message buf into a vector of bytes and a vector of handles.
    pub fn split(self) -> (Vec<u8>, Vec<Handle>) {
        (self.bytes, self.handles)
    }

    /// Ensure that the buffer has the capacity to hold at least `n_bytes` bytes.
    pub fn ensure_capacity_bytes(&mut self, n_bytes: usize) {
        ensure_capacity(&mut self.bytes, n_bytes);
    }

    /// Ensure that the buffer has the capacity to hold at least `n_handles` handles.
    pub fn ensure_capacity_handles(&mut self, n_handles: usize) {
        ensure_capacity(&mut self.handles, n_handles);
    }

    /// Ensure that at least n_bytes bytes are initialized (0 fill).
    pub fn ensure_initialized_bytes(&mut self, n_bytes: usize) {
        if n_bytes <= self.bytes.len() {
            return;
        }
        self.bytes.resize(n_bytes, 0);
    }

    /// Get a reference to the bytes of the message buffer, as a `&[u8]` slice.
    pub fn bytes(&self) -> &[u8] {
        self.bytes.as_slice()
    }

    /// The number of handles in the message buffer. Note this counts the number
    /// available when the message was received; `take_handle` does not affect
    /// the count.
    pub fn n_handles(&self) -> usize {
        self.handles.len()
    }

    /// Take the handle at the specified index from the message buffer. If the
    /// method is called again with the same index, it will return `None`, as
    /// will happen if the index exceeds the number of handles available.
    pub fn take_handle(&mut self, index: usize) -> Option<Handle> {
        self.handles.get_mut(index).and_then(|handle| {
            if handle.is_invalid() {
                None
            } else {
                Some(mem::replace(handle, Handle::invalid()))
            }
        })
    }

    /// Clear the bytes and handles contained in the buf. This will drop any
    /// contained handles, resulting in their resources being freed.
    pub fn clear(&mut self) {
        self.bytes.clear();
        self.handles.clear();
    }
}

/// A buffer for _receiving_ messages from a channel.
///
/// This differs from `MessageBuf` in that it holds `HandleInfo` with
/// extended handle information.
///
/// A `MessageBufEtc` is essentially a byte buffer and a vector of handle
/// infos, but move semantics for "taking" handles requires special handling.
///
/// Note that for sending messages to a channel, the caller manages the buffers,
/// using a plain byte slice and `Vec<HandleDisposition>`.
#[derive(Debug, Default)]
pub struct MessageBufEtc {
    bytes: Vec<u8>,
    handle_infos: Vec<HandleInfo>,
}

impl MessageBufEtc {
    /// Create a new, empty, message buffer.
    pub fn new() -> Self {
        Default::default()
    }

    /// Create a new non-empty message buffer.
    pub fn new_with(v: Vec<u8>, h: Vec<HandleInfo>) -> Self {
        Self {
            bytes: v,
            handle_infos: h,
        }
    }

    /// Splits apart the message buf into a vector of bytes and a vector of handle infos.
    pub fn split_mut(&mut self) -> (&mut Vec<u8>, &mut Vec<HandleInfo>) {
        (&mut self.bytes, &mut self.handle_infos)
    }

    /// Splits apart the message buf into a vector of bytes and a vector of handle infos.
    pub fn split(self) -> (Vec<u8>, Vec<HandleInfo>) {
        (self.bytes, self.handle_infos)
    }

    /// Ensure that the buffer has the capacity to hold at least `n_bytes` bytes.
    pub fn ensure_capacity_bytes(&mut self, n_bytes: usize) {
        ensure_capacity(&mut self.bytes, n_bytes);
    }

    /// Ensure that the buffer has the capacity to hold at least `n_handles` handle infos.
    pub fn ensure_capacity_handle_infos(&mut self, n_handle_infos: usize) {
        ensure_capacity(&mut self.handle_infos, n_handle_infos);
    }

    /// Ensure that at least n_bytes bytes are initialized (0 fill).
    pub fn ensure_initialized_bytes(&mut self, n_bytes: usize) {
        if n_bytes <= self.bytes.len() {
            return;
        }
        self.bytes.resize(n_bytes, 0);
    }

    /// Get a reference to the bytes of the message buffer, as a `&[u8]` slice.
    pub fn bytes(&self) -> &[u8] {
        self.bytes.as_slice()
    }

    /// The number of handles in the message buffer. Note this counts the number
    /// available when the message was received; `take_handle` does not affect
    /// the count.
    pub fn n_handle_infos(&self) -> usize {
        self.handle_infos.len()
    }

    /// Take the handle at the specified index from the message buffer. If the
    /// method is called again with the same index, it will return `None`, as
    /// will happen if the index exceeds the number of handles available.
    pub fn take_handle_info(&mut self, index: usize) -> Option<HandleInfo> {
        self.handle_infos.get_mut(index).and_then(|handle_info| {
            if handle_info.handle.is_invalid() {
                None
            } else {
                Some(mem::replace(
                    handle_info,
                    HandleInfo {
                        handle: Handle::invalid(),
                        object_type: ObjectType::NONE,
                        rights: Rights::NONE,
                    },
                ))
            }
        })
    }

    /// Clear the bytes and handles contained in the buf. This will drop any
    /// contained handles, resulting in their resources being freed.
    pub fn clear(&mut self) {
        self.bytes.clear();
        self.handle_infos.clear();
    }
}

fn ensure_capacity<T>(vec: &mut Vec<T>, size: usize) {
    let len = vec.len();
    if size > len {
        vec.reserve(size - len);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HandleOp, Rights, Signals};
    use std::thread;

    /* #[test]
    fn channel_basic() {
        let (p1, p2) = Channel::create();
        let mut empty = vec![];
        assert!(p1.write(b"hello", &mut empty).is_ok());
        let mut buf = MessageBuf::new();
        assert!(p2.read(&mut buf).is_ok());
        assert_eq!(buf.bytes(), b"hello");
    }

    #[test]
    fn channel_basic_etc() {
        let (p1, p2) = Channel::create();
        let mut empty = vec![];
        assert!(p1.write_etc(b"hello", &mut empty).is_ok());
        let mut buf = MessageBufEtc::new();
        assert!(p2.read_etc(&mut buf).is_ok());
        assert_eq!(buf.bytes(), b"hello");
    } */
}
