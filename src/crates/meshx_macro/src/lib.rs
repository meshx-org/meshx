// Copyright 2020 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
mod transformer;

use proc_macro::TokenStream;
use transformer::Transformer;

/// Define a meshx main function.
///
/// This attribute should be applied to the process `main` function.
/// It will take care of setting up various Fuchsia crates for the component.
/// If an async function is provided, a meshx-async Executor will be used to execute it.
///
/// Arguments:
///  - `threads` - integer worker thread count for the component. Must be >0. Default 1.
///  - `logging` - boolean toggle for whether to initialize logging (or not). Default true.
///  - `logging_tags` - optional list of string to be used as tags for logs. Default: None.
///  - `logging_minimum_severity` - optional minimum severity to be set for logs. Default: None,
///                                 the logging library will choose it (typically `info`).
///  - `logging_panic_prefix` - optional string indicating the panic message prefix to log
///
/// The main function can return either () or a Result<(), E> where E is an error type.
#[proc_macro_attribute]
pub fn main(args: TokenStream, input: TokenStream) -> TokenStream {
    Transformer::parse_main(args.into(), input.into()).unwrap().finish().into()
}