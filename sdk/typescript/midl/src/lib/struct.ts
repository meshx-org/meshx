// Copyright 2023 The MeshX Authors. All rights reserved.
// Copyright 2022 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

import { Encoder, Decoder } from "./codec"
import { deepEquals } from './hash_codes'

export abstract class Struct {
    constructor() {}

    private fields: Array<Object | null> = []

    // @override
    // int get hashCode => deepHash($fields);

    abstract $encode(encoder: Encoder, offset: number, depth: number): void

    static equal(rhs: Struct, other: Struct): boolean {
        if (rhs === other) {
            return true
        }

        //if (runtimeType != other.runtimeType) {
        //    return false;
        //}

        if (other instanceof Struct) {
            return deepEquals(rhs.fields, other.fields)
        }

        return false
    }

    toString(): string {
        return `Struct(${this.fields})`
    }
}

export type StructDecode<T> = (decoder: Decoder, offset: number, depth: number) => T
