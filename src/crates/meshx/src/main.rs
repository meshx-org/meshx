// Copyright 2023 MeshX Contributors. All rights reserved.

use fiber_kernel::Kernel;
use fiber_rust::sys;
use fiber_status as fx_status;
use std::sync::Arc;

fn main() -> Result<(), fx_status::Status> {
    // install global collector configured based on RUST_LOG env var.
    tracing_subscriber::fmt::init();

    let mut kernel = Kernel::new(|process| {
        let _vmo = process.get_vmo();
    });

    /*kernel.register_boot_process(|| {
        log::info!("boot process started");

        let root_job = unsafe { Handle::from_raw(1) };
        let job = Job::from(root_job).create_child_job().unwrap();

        let name = String::from("test");
        let (process, _vmar) = job.create_child_process(name.as_str()).unwrap();
        std::mem::forget(name);

        process.start(0, unsafe { Handle::from_raw(0) }).unwrap();
        process.start(0, unsafe { Handle::from_raw(0) }).unwrap();
    });*/

    // TODO: register the component manager process

    kernel.init();

    let kernel = Arc::new(kernel);
    sys::SYSTEM.set(kernel.clone()).unwrap();

    kernel.start();

    Ok(())
}
