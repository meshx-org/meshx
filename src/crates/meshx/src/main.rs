// Copyright 2023 MeshX Contributors. All rights reserved.
mod userboot;

use std::sync::Arc;

use fiber_kernel::Kernel;
use fiber_rust::{prelude::*, sys, Handle, Job, Process};
use fiber_status as fx_status;

use log::{debug, info};
use phf::phf_map;

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

fn main() -> Result<(), fx_status::Status> {
    env_logger::init();

    let mut kernel = Kernel::new(|process| {
        let vmo = process.get_vmo();
        let main_fn = PROCESS_DISPATCH_TABLE["+"];
        info!("result = {}", main_fn(2, 2));
    });

    kernel.register_boot_process(|| {
        info!("boot process started");

        let root_job = unsafe { Handle::from_raw(1) };
        let job = Job::from(root_job).create_child_job().unwrap();

        let name = String::from("test");
        let (process, _vmar) = job.create_child_process(name.as_str()).unwrap();
        std::mem::forget(name);

        process.start(0, unsafe { Handle::from_raw(0) });
        process.start(0, unsafe { Handle::from_raw(0) });
    });

    // TODO: register the component manager process

    let kernel = Arc::new(kernel);
    sys::SYSTEM.set(kernel.clone()).unwrap();

    kernel.run_root();

    let plus = PROCESS_DISPATCH_TABLE["+"];
    info!("2 + 3 = {}", plus(2, 3));

    let minus = PROCESS_DISPATCH_TABLE["-"];
    info!("2 - 3 = {}", minus(2, 3));

    Ok(())
}
