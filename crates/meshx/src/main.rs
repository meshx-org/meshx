// Copyright 2022 MeshX Contributors. All rights reserved.

use fiber_kernel::Kernel;
use fiber_rust::{prelude::*, sys, Handle, Job, Process};

fn main() {
    let kernel = Box::new(Kernel::new());
    kernel.init();

    sys::SYSTEM.set(kernel);

    let root_job = unsafe { Handle::from_raw(0) };

    let job = Job::from(root_job).create_child_job().unwrap();
    let process = job.create_child_process("test".to_string().as_bytes()).unwrap();
    

    println!("{}", "Hello, world!");
}
