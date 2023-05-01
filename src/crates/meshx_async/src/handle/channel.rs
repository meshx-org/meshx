// Copyright 2023 MeshX Contributors. All rights reserved.
// Copyright 2018 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::{OnSignals, RWHandle, ReadableHandle as _};
use fiber_rust::{self as fx, AsHandleRef, MessageBuf, MessageBufEtc};
use futures::ready;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// An I/O object representing a `Channel`.
pub struct Channel(RWHandle<fx::Channel>);

impl AsRef<fx::Channel> for Channel {
    fn as_ref(&self) -> &fx::Channel {
        self.0.get_ref()
    }
}

impl AsHandleRef for Channel {
    fn as_handle_ref(&self) -> fx::HandleRef<'_> {
        self.0.get_ref().as_handle_ref()
    }
}

impl From<Channel> for fx::Channel {
    fn from(channel: Channel) -> fx::Channel {
        channel.0.into_inner()
    }
}

impl Channel {
    /// Creates a new `Channel` from a previously-created `zx::Channel`.
    pub fn from_channel(channel: fx::Channel) -> Result<Self, fx::Status> {
        Ok(Channel(RWHandle::new(channel)?))
    }

    /// Consumes `self` and returns the underlying `zx::Channel`.
    pub fn into_zx_channel(self) -> fx::Channel {
        self.0.into_inner()
    }

    /// Returns true if the channel received the `OBJECT_PEER_CLOSED` signal.
    pub fn is_closed(&self) -> bool {
        self.0.is_closed()
    }

    /// Returns a future that completes when `is_closed()` is true.
    pub fn on_closed<'a>(&'a self) -> OnSignals<'a> {
        self.0.on_closed()
    }

    /// Receives a message on the channel and registers this `Channel` as
    /// needing a read on receiving a `zx::Status::SHOULD_WAIT`.
    ///
    /// Identical to `recv_from` except takes separate bytes and handles buffers
    /// rather than a single `MessageBuf`.
    pub fn read(
        &self,
        cx: &mut Context<'_>,
        bytes: &mut Vec<u8>,
        handles: &mut Vec<fx::Handle>,
    ) -> Poll<Result<(), fx::Status>> {
        ready!(self.0.poll_readable(cx))?;
        let res = self.0.get_ref().read_split(bytes, handles);

        if res == Err(fx::Status::SHOULD_WAIT) {
            self.0.need_readable(cx)?;
            return Poll::Pending;
        }

        Poll::Ready(res)
    }

    /// Receives a message on the channel and registers this `Channel` as
    /// needing a read on receiving a `zx::Status::SHOULD_WAIT`.
    ///
    /// Identical to `recv_etc_from` except takes separate bytes and handles
    /// buffers rather than a single `MessageBufEtc`.
    pub fn read_etc(
        &self,
        cx: &mut Context<'_>,
        bytes: &mut Vec<u8>,
        handles: &mut Vec<fx::HandleInfo>,
    ) -> Poll<Result<(), fx::Status>> {
        ready!(self.0.poll_readable(cx))?;
        let res = self.0.get_ref().read_etc_split(bytes, handles);

        if res == Err(fx::Status::SHOULD_WAIT) {
            self.0.need_readable(cx)?;
            return Poll::Pending;
        }

        Poll::Ready(res)
    }

    /// Receives a message on the channel and registers this `Channel` as
    /// needing a read on receiving a `zx::Status::SHOULD_WAIT`.
    pub fn recv_from(&self, cx: &mut Context<'_>, buf: &mut MessageBuf) -> Poll<Result<(), fx::Status>> {
        let (bytes, handles) = buf.split_mut();
        self.read(cx, bytes, handles)
    }

    /// Receives a message on the channel and registers this `Channel` as
    /// needing a read on receiving a `zx::Status::SHOULD_WAIT`.
    pub fn recv_etc_from(&self, cx: &mut Context<'_>, buf: &mut MessageBufEtc) -> Poll<Result<(), fx::Status>> {
        let (bytes, handles) = buf.split_mut();
        self.read_etc(cx, bytes, handles)
    }

    /// Creates a future that receive a message to be written to the buffer
    /// provided.
    ///
    /// The returned future will return after a message has been received on
    /// this socket and been placed into the buffer.
    pub fn recv_msg<'a>(&'a self, buf: &'a mut MessageBuf) -> RecvMsg<'a> {
        RecvMsg { channel: self, buf }
    }

    /// Creates a future that receive a message to be written to the buffer
    /// provided.
    ///
    /// The returned future will return after a message has been received on
    /// this socket and been placed into the buffer.
    pub fn recv_etc_msg<'a>(&'a self, buf: &'a mut MessageBufEtc) -> RecvEtcMsg<'a> {
        RecvEtcMsg { channel: self, buf }
    }

    /// Writes a message into the channel.
    pub fn write(&self, bytes: &[u8], handles: &mut [fx::Handle]) -> Result<(), fx::Status> {
        self.0.get_ref().write(bytes, handles)
    }

    /// Writes a message into the channel.
    pub fn write_etc(&self, bytes: &[u8], handles: &mut [fx::HandleDisposition<'_>]) -> Result<(), fx::Status> {
        self.0.get_ref().write_etc(bytes, handles)
    }
}

impl fmt::Debug for Channel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.get_ref().fmt(f)
    }
}

/// A future used to receive a message from a channel.
///
/// This is created by the `Channel::recv_msg` method.
#[must_use = "futures do nothing unless polled"]
pub struct RecvMsg<'a> {
    channel: &'a Channel,
    buf: &'a mut MessageBuf,
}

impl<'a> Future for RecvMsg<'a> {
    type Output = Result<(), fx::Status>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;
        this.channel.recv_from(cx, this.buf)
    }
}

/// A future used to receive a message from a channel.
///
/// This is created by the `Channel::recv_etc_msg` method.
#[must_use = "futures do nothing unless polled"]
pub struct RecvEtcMsg<'a> {
    channel: &'a Channel,
    buf: &'a mut MessageBufEtc,
}

impl<'a> Future for RecvEtcMsg<'a> {
    type Output = Result<(), fx::Status>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;
        this.channel.recv_etc_from(cx, this.buf)
    }
}
