// Copyright 2022 MeshX Contributors. All rights reserved.

use std::collections::HashMap;

use fiber_kernel::Kernel;
use fiber_rust::{prelude::*, sys, Handle, Job, Process};
use log::{debug, info};
use phf::phf_map;
use std::io::Write;

fn baz() {
    println!("fn baz");
}

fn plus(a: i32, b: i32) -> i32 {
    a + b
}

fn minus(a: i32, b: i32) -> i32 {
    a - b
}

static PROCESS_DISPATCH_TABLE: phf::Map<&'static str, fn(i32, i32) -> i32> = phf_map! {
    "+" => plus,
    "-" => minus,
};

fn main() {
    env_logger::init();

    let kernel = Kernel::new(|process| {
        // TODO(szkabaroli): parse vmo 
        let vmo = process.get_vmo();
        // let program = parse_program();
        let main_fn = PROCESS_DISPATCH_TABLE["+"];
        info!("result = {}", main_fn(2, 2));
    });

    kernel.init();

    sys::SYSTEM.set(Box::new(kernel));

    let root_job = unsafe { Handle::from_raw(0) };

    let job = Job::from(root_job).create_child_job().unwrap();
    let process = job.create_child_process("test".to_string().as_bytes()).unwrap();

    sys::fx_process_start(0, 0, 0);
    sys::fx_process_start(0, 0, 0);

    let plus = PROCESS_DISPATCH_TABLE["+"];
    debug!("2 + 3 = {}", plus(2, 3));

    let minus = PROCESS_DISPATCH_TABLE["-"];
    info!("2 - 3 = {}", minus(2, 3));
}
