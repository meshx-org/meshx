// Copyright 2023 MeshX Contributors. All rights reserved.
// Copyright 2019 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! Epitaph support for Channel and AsyncChannel.
//!
use {
    crate::{
        encoding::{self, DynamicFlags, EpitaphBody, TransactionHeader, TransactionMessage, TransactionMessageType},
        error::Error,
        AsyncChannel, Channel, HandleDisposition,
    },
    fiber_status as fx_status,
};

/// Extension trait that provides Channel-like objects with the ability to send a FIDL epitaph.
pub trait ChannelEpitaphExt {
    /// Consumes the channel and writes an epitaph.
    fn close_with_epitaph(self, status: fx_status::Status) -> Result<(), Error>;
}

impl ChannelEpitaphExt for Channel {
    fn close_with_epitaph(self, status: fx_status::Status) -> Result<(), Error> {
        write_epitaph_impl(&self, status)
    }
}

impl ChannelEpitaphExt for AsyncChannel {
    fn close_with_epitaph(self, status: fx_status::Status) -> Result<(), Error> {
        write_epitaph_impl(&self, status)
    }
}

pub(crate) trait ChannelLike {
    fn write_etc<'a>(&self, bytes: &[u8], handles: &mut Vec<HandleDisposition<'a>>) -> Result<(), fx_status::Status>;
}

impl ChannelLike for Channel {
    fn write_etc<'a>(&self, bytes: &[u8], handles: &mut Vec<HandleDisposition<'a>>) -> Result<(), fx_status::Status> {
        self.write_etc(bytes, handles)
    }
}

impl ChannelLike for AsyncChannel {
    fn write_etc<'a>(&self, bytes: &[u8], handles: &mut Vec<HandleDisposition<'a>>) -> Result<(), fx_status::Status> {
        self.write_etc(bytes, handles)
    }
}

pub(crate) fn write_epitaph_impl<T: ChannelLike>(channel: &T, status: fx_status::Status) -> Result<(), Error> {
    let msg = TransactionMessage {
        header: TransactionHeader::new(0, encoding::EPITAPH_ORDINAL, DynamicFlags::empty()),
        body: &EpitaphBody { error: status },
    };
    encoding::with_tls_encoded::<TransactionMessageType<EpitaphBody>, (), false>(msg, |bytes, handles| {
        channel.write_etc(bytes, handles).map_err(Error::ServerEpitaphWrite)
    })
}
