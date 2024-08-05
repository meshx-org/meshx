#![feature(vec_into_raw_parts)]

use builtin_environment::BuiltinEnvironment;
use builtin_environment::BuiltinEnvironmentBuilder;
use cm_config::RuntimeConfig;
use log::info;
use std::ffi::CString;
use std::panic;
use std::string::String;
use anyhow::Error;

mod routing;
mod startup;
mod builtin;
mod types;
mod model;
mod clonable_error;
mod builtin_environment;

fn write(_value: String) {
    unsafe {
        // let (data, len, _) = _value.into_raw_parts();
        let c_string: CString = CString::new(_value).expect("CString::new failed");
        fiber_sys::fx_debug(c_string.as_ptr() , c_string.count_bytes());
    }
}

//custom_print::define_macros!({ cprint, cprintln }, concat, self::write);
//custom_print::define_macros!({ eprint, eprintln, dbg }, concat, self::write);
custom_print::define_init_panic_hook!(concat, self::write);

//macro_rules! println { ($($args:tt)*) => { cprintln!($($args)*); } }
//macro_rules! print { ($($args:tt)*) => { cprint!($($args)*); } }

fn main() {
    fiber_wasi_polyfill::init();
    init_panic_hook();
    println!("env={:?}", std::env::args());

    let args = startup::Arguments::from_args().unwrap_or_else(|err| panic!("{}\n{}", err, startup::Arguments::usage()));

    let mut executor = meshx_async::LocalExecutor::new();

    info!("Component manager is starting up...");
    if args.boot {
        info!("Component manager was started with boot defaults");
    }

    let run_root_fut = async move {
        println!("Hello executor");
    };

    return executor.run_singlethreaded(run_root_fut);
}

async fn build_environment(config: RuntimeConfig/* , bootfs_svc: Option<BootfsSvc>*/) -> Result<BuiltinEnvironment, Error> {
    let mut builder = BuiltinEnvironmentBuilder::new()
        .set_runtime_config(config)
        .add_builtin_runner()?
        .include_namespace_resolvers();

    //if let Some(bootfs_svc) = bootfs_svc {
    //    builder = builder.set_bootfs_svc(bootfs_svc);
    //}

    builder.build().await
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
