#![feature(vec_into_raw_parts)]

use std::ffi::CString;
use std::sync::Mutex;

use fiber_sys::fx_channel_read;
use fiber_sys::fx_proc_args_t;
use fiber_sys::fx_handle_t;
use fiber_sys::FX_HANDLE_INVALID;
use fiber_types::*;

pub type Size = usize;
pub type Exitcode = u32;

/// No error occurred. System call completed successfully.
pub const ERRNO_SUCCESS: i32 = 0;

#[repr(transparent)]
#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Errno(u16);

static STARTUP_HANDLES_LOCK: Mutex<()> = Mutex::new(());
static mut STARTUP_HANDLES_NUM: u32 = 0;
static mut STARTUP_HANDLES: *mut fx_handle_t = std::ptr::null_mut();
static mut STARTUP_HANDLES_INFO: *mut u32 = std::ptr::null_mut();

fn shave_front() {
    unsafe {
        while STARTUP_HANDLES_NUM > 0 && *STARTUP_HANDLES == FX_HANDLE_INVALID {
            STARTUP_HANDLES_NUM -= 1;
            STARTUP_HANDLES = STARTUP_HANDLES.add(1);
            STARTUP_HANDLES_INFO = STARTUP_HANDLES_INFO.add(1);
        }
    }
}

fn shave_back() {
    unsafe {
        while STARTUP_HANDLES_NUM > 0 && *STARTUP_HANDLES.add(STARTUP_HANDLES_NUM as usize - 1) == FX_HANDLE_INVALID {
            STARTUP_HANDLES_NUM -= 1;
        }
    }
}

// This function is called only once at startup, so it doesn't need locking.
pub fn libc_startup_handles_init(nhandles: u32, handles: &mut [fx_handle_t], handle_info: &mut [u32]) {
    unsafe {
        STARTUP_HANDLES_NUM = nhandles;
        STARTUP_HANDLES = handles.as_mut_ptr();
        STARTUP_HANDLES_INFO = handle_info.as_mut_ptr();
    }
    shave_front();
    shave_back();
}

pub fn fx_take_startup_handle(hnd_info: u32) -> fx_handle_t {
    let _lock = STARTUP_HANDLES_LOCK.lock().unwrap();
    unsafe {
        let mut result = FX_HANDLE_INVALID;
        for i in 0..STARTUP_HANDLES_NUM as usize {
            if *STARTUP_HANDLES_INFO.add(i) == hnd_info {
                result = *STARTUP_HANDLES.add(i);
                *STARTUP_HANDLES.add(i) = FX_HANDLE_INVALID;
                *STARTUP_HANDLES_INFO.add(i) = 0;
                if i == 0 {
                    shave_front();
                } else if i == STARTUP_HANDLES_NUM as usize - 1 {
                    shave_back();
                }
                break;
            }
        }
        result
    }
}

#[link(wasm_import_module = "fiber")]
extern "C" {
    /// WASI entry
    fn _start();
}

fn processargs_message_size(channel: fx_handle_t, actual_bytes: &mut u32, actual_handles: &mut u32) -> fx_status_t {
    // The equivalent of _zx_channel_read with NULL for bytes and handles
    let mut status = unsafe {
        fx_channel_read(
            channel,
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            0,
            0,
            actual_bytes,
            actual_handles,
        )
    };

    if status == FX_ERR_BUFFER_TOO_SMALL {
        status = FX_OK;
    }

    status
}

#[derive(Debug, Default)]
struct StartParams {
    procargs: fx_proc_args_t,
    nbytes: u32,
    nhandles: u32,
}

unsafe fn start_main(p: StartParams) {
    let argc = p.procargs.args_num;
    let envc = p.procargs.environ_num;
    let namec = p.procargs.names_num;

    let string = format!("test");
    // let s = string.into_bytes();

    let c_string = CString::new(string).expect("CString::new failed");
     
    fiber_sys::fx_debug(c_string.as_ptr(), c_string.count_bytes());

    _start();
}

// Defined in Scrt1.o, linked into main executable.
#[no_mangle]
#[export_name = "main"]
pub unsafe fn main(bootstrap: fx_handle_t) {
    let mut p = StartParams::default();
    // extract process startup information from channel in arg
    //struct start_params p = {.main = main, .utc_reference = ZX_HANDLE_INVALID};
    let status: fx_status_t = processargs_message_size(bootstrap, &mut p.nbytes, &mut p.nhandles);

    /*if (status == ZX_OK) && p.nbytes && p.nhandles {
        PROCESSARGS_BUFFER(buffer, p.nbytes);
        zx_handle_t handles[p.nhandles];
        p.buffer = buffer;
        p.handles = handles;
        status = processargs_read(bootstrap, buffer, p.nbytes, handles, p.nhandles, &p.procargs,
                                &p.handle_info);
        if (status != ZX_OK) {
        CRASH_WITH_UNIQUE_BACKTRACE();
        }
        _zx_handle_close(bootstrap);
        zx_handle_t main_thread_handle = ZX_HANDLE_INVALID;
        processargs_extract_handles(p.nhandles, handles, p.handle_info, &__zircon_process_self, &__zircon_job_default, &__zircon_vmar_root_self, &main_thread_handle, &p.utc_reference);
    }*/

    start_main(p)
}

/// Read command-line argument data.
/// The size of the array should match that returned by `args_sizes_get`.
/// Each argument is expected to be `\0` terminated.
#[no_mangle]
pub unsafe fn args_get(argv: *mut *mut u8, argv_buf: *mut u8) -> i32 {
    ERRNO_SUCCESS
}

/// Return command-line argument data sizes.
///
/// ## Return
///
/// Returns the number of arguments and the size of the argument string
/// data, or an error.
#[no_mangle]
pub unsafe fn args_sizes_get(argc: *mut u8, argc_buf: *mut u8) -> i32 {
    ERRNO_SUCCESS
}

/// Terminate the process normally. An exit code of 0 indicates successful
/// termination of the program. The meanings of other values is dependent on
/// the environment.
///
/// ## Parameters
///
/// * `rval` - The exit code returned by the process.
#[no_mangle]
pub unsafe fn proc_exit(rval: Exitcode) {}

/// Read environment variable data.
/// The sizes of the buffers should match that returned by `environ_sizes_get`.
/// Key/value pairs are expected to be joined with `=`s, and terminated with `\0`s.
#[no_mangle]
pub unsafe fn environ_get(environ: *mut *mut u8, environ_buf: *mut u8) -> i32 {
    ERRNO_SUCCESS
}

/// Return environment variable data sizes.
///
/// ## Return
///
/// Returns the number of environment variable arguments and the size of the
/// environment variable data.
#[no_mangle]
pub unsafe fn environ_sizes_get(envc: *mut u8, envc_buf: *mut u8) -> i32 {
    ERRNO_SUCCESS
}

/// Temporarily yield execution of the calling thread.
/// Note: This is similar to `sched_yield` in POSIX.
#[no_mangle]
pub unsafe fn sched_yield() -> i32 {
    ERRNO_SUCCESS
}

/// Write high-quality random data into a buffer.
/// This function blocks when the implementation is unable to immediately
/// provide sufficient high-quality random data.
/// This function may execute slowly, so when large mounts of random data are
/// required, it's advisable to use this function to seed a pseudo-random
/// number generator, rather than to provide the random data directly.
///
/// ## Parameters
///
/// * `buf` - The buffer to fill with random data.
#[no_mangle]
pub unsafe fn random_get(buf: *mut u8, buf_len: Size) -> i32 {
    ERRNO_SUCCESS
}

/// Write to a file descriptor.
/// Note: This is similar to `writev` in POSIX.
///
/// ## Parameters
///
/// * `iovs` - List of scatter/gather vectors from which to retrieve data.
#[no_mangle]
pub unsafe fn fd_write(fd: i32, iovs_ptr: *const i32, iovs_len: i32, rp0: *mut i32) -> i32 {
    ERRNO_SUCCESS
}

/// Return the time value of a clock.
/// Note: This is similar to `clock_gettime` in POSIX.
///
/// ## Parameters
///
/// * `id` - The clock for which to return the time.
/// * `precision` - The maximum lag (exclusive) that the returned time value may have, compared to its actual value.
///
/// ## Return
///
/// The time value of the clock.
#[no_mangle]
pub unsafe fn clock_time_get(id: i32, precision: i64, timestamp: *mut i32) -> i32 {
    ERRNO_SUCCESS
}
