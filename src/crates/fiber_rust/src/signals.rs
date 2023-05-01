// Copyright 2023 MeshX Contributors. All rights reserved.
// Copyright 2018 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! Type-safe bindings for Zircon signals.
use bitflags::bitflags;
use fiber_sys::*;

bitflags! {
    /// Signals that can be waited upon.
    ///
    /// See
    /// [Objects and signals](https://fuchsia.dev/fuchsia-src/concepts/kernel/concepts#objects_and_signals)
    /// in the Zircon kernel documentation. Note: the names of signals are still in flux.
    #[repr(transparent)]
    pub struct Signals: fx_signals_t {
        const NONE          = FX_SIGNAL_NONE;
        const OBJECT_ALL    = FX_OBJECT_SIGNAL_ALL;
        const USER_ALL      = FX_USER_SIGNAL_ALL;
        const OBJECT_0      = FX_OBJECT_SIGNAL_0;
        const OBJECT_1      = FX_OBJECT_SIGNAL_1;
        const OBJECT_2      = FX_OBJECT_SIGNAL_2;
        const OBJECT_3      = FX_OBJECT_SIGNAL_3;
        const OBJECT_4      = FX_OBJECT_SIGNAL_4;
        const OBJECT_5      = FX_OBJECT_SIGNAL_5;
        const OBJECT_6      = FX_OBJECT_SIGNAL_6;
        const OBJECT_7      = FX_OBJECT_SIGNAL_7;
        const OBJECT_8      = FX_OBJECT_SIGNAL_8;
        const OBJECT_9      = FX_OBJECT_SIGNAL_9;
        const OBJECT_10     = FX_OBJECT_SIGNAL_10;
        const OBJECT_11     = FX_OBJECT_SIGNAL_11;
        const OBJECT_12     = FX_OBJECT_SIGNAL_12;
        const OBJECT_13     = FX_OBJECT_SIGNAL_13;
        const OBJECT_14     = FX_OBJECT_SIGNAL_14;
        const OBJECT_15     = FX_OBJECT_SIGNAL_15;
        const OBJECT_16     = FX_OBJECT_SIGNAL_16;
        const OBJECT_17     = FX_OBJECT_SIGNAL_17;
        const OBJECT_18     = FX_OBJECT_SIGNAL_18;
        const OBJECT_19     = FX_OBJECT_SIGNAL_19;
        const OBJECT_20     = FX_OBJECT_SIGNAL_20;
        const OBJECT_21     = FX_OBJECT_SIGNAL_21;
        const OBJECT_22     = FX_OBJECT_SIGNAL_22;
        const OBJECT_HANDLE_CLOSED = FX_OBJECT_HANDLE_CLOSED;
        const USER_0        = FX_USER_SIGNAL_0;
        const USER_1        = FX_USER_SIGNAL_1;
        const USER_2        = FX_USER_SIGNAL_2;
        const USER_3        = FX_USER_SIGNAL_3;
        const USER_4        = FX_USER_SIGNAL_4;
        const USER_5        = FX_USER_SIGNAL_5;
        const USER_6        = FX_USER_SIGNAL_6;
        const USER_7        = FX_USER_SIGNAL_7;
        const OBJECT_READABLE    = FX_OBJECT_READABLE;
        const OBJECT_WRITABLE    = FX_OBJECT_WRITABLE;
        const OBJECT_PEER_CLOSED = FX_OBJECT_PEER_CLOSED;
        // Cancelation (handle was closed while waiting with it)
        const HANDLE_CLOSED = FX_SIGNAL_HANDLE_CLOSED;
        // Event
        const EVENT_SIGNALED = FX_EVENT_SIGNALED;
        // EventPair
        const EVENTPAIR_SIGNALED = FX_EVENTPAIR_SIGNALED;
        const EVENTPAIR_CLOSED   = FX_EVENTPAIR_CLOSED;
        // Task signals (process, thread, job)
        const TASK_TERMINATED = FX_TASK_TERMINATED;
        // Channel
        const CHANNEL_READABLE    = FX_CHANNEL_READABLE;
        const CHANNEL_WRITABLE    = FX_CHANNEL_WRITABLE;
        const CHANNEL_PEER_CLOSED = FX_CHANNEL_PEER_CLOSED;
        // Clock
        const CLOCK_STARTED       = FX_CLOCK_STARTED;
        // Socket
        const SOCKET_READABLE            = FX_SOCKET_READABLE;
        const SOCKET_WRITABLE            = FX_SOCKET_WRITABLE;
        const SOCKET_PEER_CLOSED         = FX_SOCKET_PEER_CLOSED;
        const SOCKET_PEER_WRITE_DISABLED = FX_SOCKET_PEER_WRITE_DISABLED;
        const SOCKET_WRITE_DISABLED      = FX_SOCKET_WRITE_DISABLED;
        const SOCKET_READ_THRESHOLD      = FX_SOCKET_READ_THRESHOLD;
        const SOCKET_WRITE_THRESHOLD     = FX_SOCKET_WRITE_THRESHOLD;
        // Job
        const JOB_TERMINATED   = FX_JOB_TERMINATED;
        const JOB_NO_JOBS      = FX_JOB_NO_JOBS;
        const JOB_NO_PROCESSES = FX_JOB_NO_PROCESSES;
        // Process
        const PROCESS_TERMINATED = FX_PROCESS_TERMINATED;
        // Thread
        const THREAD_TERMINATED = FX_THREAD_TERMINATED;
        const THREAD_RUNNING    = FX_THREAD_RUNNING;
        const THREAD_SUSPENDED  = FX_THREAD_SUSPENDED;
        // Log
        const LOG_READABLE = FX_LOG_READABLE;
        const LOG_WRITABLE = FX_LOG_WRITABLE;
        // Timer
        const TIMER_SIGNALED = FX_TIMER_SIGNALED;
        // Vmo
        const VMO_ZERO_CHILDREN = FX_VMO_ZERO_CHILDREN;
    }
}