// Copyright 2023 The MeshX Authors. All rights reserved.
// Copyright 2022 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

export enum WireFormat {
    v2,
}

export const kWireFormatDefault = WireFormat.v2;
export const kEnvelopeInlineMarker = 1;
export const kEnvelopeOutOfLineMarker = 0;