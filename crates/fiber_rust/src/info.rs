// Copyright 2018 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//! Type-safe bindings for Zircon object information.
use fiber_sys as sys;
use std::ops::Deref;

use crate::assoc_values;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct Topic(sys::fx_object_info_topic_t);

impl Deref for Topic {
    type Target = sys::fx_object_info_topic_t;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A query to get info about a zircon object.
///
/// Safety: `TOPIC` must correspond to a valid `zx_object_get_info` topic,
/// and `InfoTy` must be a type that can be safely replaced with the byte
/// representation of the associated `zx_object_get_info` buffer type.
pub unsafe trait ObjectQuery {
    /// A `Topic` identifying this query.
    const TOPIC: Topic;
    /// The datatype returned by this query.
    type InfoTy;
}

assoc_values!(Topic, [
    NONE = sys::FX_INFO_NONE;
    HANDLE_VALID = sys::FX_INFO_HANDLE_VALID;
    HANDLE_BASIC = sys::FX_INFO_HANDLE_BASIC;
]);
