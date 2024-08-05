#![feature(vec_into_raw_parts)]
#![allow(warnings)]

use std::ffi::CString;
use std::ops::Range;
use std::{cell::RefCell, ffi::c_void};

use fiber_sys::{
    fx_channel_read, fx_handle_t, fx_proc_args_t, fx_status_t, FX_ERR_BUFFER_TOO_SMALL, FX_ERR_INVALID_ARGS,
    FX_HANDLE_INVALID, FX_OK, FX_PROCARGS_PROTOCOL, FX_PROCARGS_VERSION, PA_JOB_DEFAULT, PA_PROC_SELF,
};

#[cfg(target_arch = "wasm32")]
mod wasi;
#[cfg(not(all(target_arch = "wasm32")))]
mod wasi_mock;
#[cfg(not(all(target_arch = "wasm32")))]
use wasi_mock as wasi;

//use environment::*;
use args::*;
use wasi_helpers::*;

//mod environment;
mod args;
mod wasi_helpers;

#[cfg(target_arch = "wasm32")]
#[cfg(feature = "report_wasi_calls")]
use ic_cdk::api::instruction_counter as ic_instruction_counter;
#[cfg(not(all(target_arch = "wasm32")))]
pub fn ic_instruction_counter() -> u64 {
    0
}

#[cfg(target_arch = "wasm32")]
fn ic_time() -> u64 {
    0
}
#[cfg(not(all(target_arch = "wasm32")))]
fn ic_time() -> u64 {
    use std::time::UNIX_EPOCH;

    std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}

#[cfg(target_arch = "wasm32")]
fn ic_print(value: &str) {
    let ptr = value.as_ptr() as *mut u8;
    let len = value.len();
    unsafe { fx_debug(ptr, len) };
}
#[cfg(not(all(target_arch = "wasm32")))]
fn ic_print(value: &str) {
    println!("{}", value);
}

#[allow(clippy::missing_safety_doc)]
pub unsafe fn forward_to_debug(iovs: *const wasi::Ciovec, len: i32, res: *mut wasi::Size) -> i32 {
    let iovs = std::slice::from_raw_parts(iovs, len as usize);

    let mut written = 0;

    for iov in iovs {
        let buf = std::slice::from_raw_parts(iov.buf, iov.buf_len);
        let str = std::str::from_utf8(buf).unwrap_or("");
        ic_print(str);
        written += iov.buf_len;
    }

    *res = written;

    wasi::ERRNO_SUCCESS.raw() as i32
}

thread_local! {
    // static RNG : RefCell<rand::rngs::StdRng> = RefCell::new(rand::rngs::StdRng::from_seed([0;32]));

    //pub static FS: RefCell<FileSystem> = RefCell::new(
    //    FileSystem::new(Box::new(DummyStorage::new())).unwrap()
    //);

    static ARGS: RefCell<Args> = RefCell::new(Args::new());
}

#[allow(unused_macros)]
macro_rules! debug_instructions {
    ($fn_name:literal) => {
        ic_print(&format!("\t{}", $fn_name))
    };
    ($fn_name:literal, $sresult:expr, $stime:expr) => {
        let etime = ic_instruction_counter();

        ic_print(&format!(
            "\t{} -> {}\tinstructions:\t{}\t",
            $fn_name,
            $sresult,
            etime - ($stime)
        ))
    };
    ($fn_name:literal, $sresult:expr, $stime:expr, $params:expr) => {
        let etime = ic_instruction_counter();

        ic_print(&format!(
            "\t{} -> {}\tinstructions:\t{}\tparameters:\t{}",
            $fn_name,
            $sresult,
            etime - ($stime),
            format!($params)
        ))
    };
}

#[no_mangle]
#[inline(never)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __fx_custom_fd_write(
    fd: i32,
    iovs: *const wasi::Ciovec,
    len: i32,
    res: *mut wasi::Size,
) -> i32 {
    #[cfg(feature = "report_wasi_calls")]
    let start = ic_instruction_counter();

    let result = if fd < 3 {
        forward_to_debug(iovs, len, res)
    } else {
        wasi::ERRNO_SUCCESS.raw() as i32
    };

    #[cfg(feature = "report_wasi_calls")]
    debug_instructions!("__fx_custom_fd_write", result, start, "fd={fd:?} len={len:?}");

    result
}

#[no_mangle]
#[inline(never)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __fx_custom_fd_read(
    fd: i32,
    iovs: *const wasi::Ciovec,
    len: i32,
    res: *mut wasi::Size,
) -> i32 {
    #[cfg(feature = "report_wasi_calls")]
    let start = ic_instruction_counter();

    // for now we don't support reading from the standard streams
    if fd < 3 {
        return wasi::ERRNO_INVAL.raw() as i32;
    }

    let result = wasi::ERRNO_SUCCESS.raw() as i32;

    #[cfg(feature = "report_wasi_calls")]
    debug_instructions!("__fx_custom_fd_read", result, start, "fd={fd:?} len={len:?}");

    result
}

#[no_mangle]
#[inline(never)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __fx_custom_fd_pwrite(
    fd: i32,
    iovs: *const wasi::Ciovec,
    len: i32,
    offset: i64,
    res: *mut wasi::Size,
) -> i32 {
    #[cfg(feature = "report_wasi_calls")]
    let start = ic_instruction_counter();

    let result = if fd < 3 {
        forward_to_debug(iovs, len, res)
    } else {
        wasi::ERRNO_SUCCESS.raw() as i32
    };

    #[cfg(feature = "report_wasi_calls")]
    debug_instructions!("__fx_custom_fd_pwrite", result, start, "fd={fd:?} len={len:?}");

    result
}

#[no_mangle]
#[inline(never)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __fx_custom_fd_pread(
    fd: i32,
    iovs: *const wasi::Ciovec,
    len: i32,
    offset: i64,
    res: *mut wasi::Size,
) -> i32 {
    #[cfg(feature = "report_wasi_calls")]
    let start = ic_instruction_counter();

    // for now we don't support reading from the standard streams
    if fd < 3 {
        return wasi::ERRNO_INVAL.raw() as i32;
    }

    let result = wasi::ERRNO_SUCCESS.raw() as i32;

    #[cfg(feature = "report_wasi_calls")]
    debug_instructions!(
        "__fx_custom_fd_pread",
        result,
        start,
        "fd={fd:?} len={len:?} offset={offset:?}"
    );

    result
}

#[no_mangle]
#[inline(never)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __fx_custom_fd_seek(fd: i32, delta: i64, whence: i32, res: *mut wasi::Filesize) -> i32 {
    #[cfg(feature = "report_wasi_calls")]
    let start = ic_instruction_counter();

    // standart streams not supported
    if fd < 3 {
        return wasi::ERRNO_INVAL.raw() as i32;
    }

    let result = wasi::ERRNO_SUCCESS.raw() as i32;

    #[cfg(feature = "report_wasi_calls")]
    debug_instructions!(
        "__fx_custom_fd_seek",
        result,
        start,
        "fd={fd:?} delta={delta:?} whence={whence:?}"
    );

    result
}

#[no_mangle]
#[inline(never)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __fx_custom_path_open(
    parent_fd: i32,
    dirflags: i32,
    path: *const u8,
    path_len: i32,

    oflags: i32,
    fs_rights_base: i64,
    fs_rights_inheriting: i64,

    fdflags: i32,
    res: *mut i32,
) -> i32 {
    #[cfg(feature = "report_wasi_calls")]
    let start = ic_instruction_counter();

    // dirflags contains the information on whether to follow the symlinks,
    // the symlinks are not supported yet by the file system
    prevent_elimination(&[dirflags]);
    let file_name = get_file_name(path, path_len as wasi::Size);

    let result = wasi::ERRNO_SUCCESS.raw() as i32;

    #[cfg(feature = "report_wasi_calls")]
    debug_instructions!(
        "__fx_custom_path_open",
        result,
        start,
        "parent_fd={parent_fd:?} file_name={file_name:?} oflags={oflags}"
    );

    result
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __fx_custom_fd_close(fd: i32) -> i32 {
    #[cfg(feature = "report_wasi_calls")]
    let start = ic_instruction_counter();

    let result = wasi::ERRNO_SUCCESS.raw() as i32;

    #[cfg(feature = "report_wasi_calls")]
    debug_instructions!("__fx_custom_fd_close", result, start, "fd={fd:?}");

    result
}

#[no_mangle]
#[inline(never)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __fx_custom_fd_filestat_get(fd: i32, ret_val: *mut wasi::Filestat) -> i32 {
    #[cfg(feature = "report_wasi_calls")]
    let start = ic_instruction_counter();

    let result = wasi::ERRNO_SUCCESS.raw() as i32;

    #[cfg(feature = "report_wasi_calls")]
    debug_instructions!("__fx_custom_fd_filestat_get", result, start, "fd={fd:?}");

    result
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __fx_custom_fd_sync(fd: i32) -> i32 {
    #[cfg(feature = "report_wasi_calls")]
    let start = ic_instruction_counter();

    prevent_elimination(&[fd]);

    let result = wasi::ERRNO_SUCCESS.raw() as i32;

    #[cfg(feature = "report_wasi_calls")]
    debug_instructions!("__fx_custom_fd_sync", result, start, "fd={fd:?}");

    result
}

#[no_mangle]
#[inline(never)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __fx_custom_fd_tell(fd: i32, res: *mut wasi::Filesize) -> i32 {
    #[cfg(feature = "report_wasi_calls")]
    let start = ic_instruction_counter();

    // standard streams not supported
    if fd < 3 {
        return wasi::ERRNO_INVAL.raw() as i32;
    }

    let result = wasi::ERRNO_SUCCESS.raw() as i32;

    #[cfg(feature = "report_wasi_calls")]
    debug_instructions!("__fx_custom_fd_tell", result, start, "{fd:?} -> {res:?}");

    result
}

#[no_mangle]
#[inline(never)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __fx_custom_fd_prestat_get(fd: i32, res: *mut wasi::Prestat) -> i32 {
    #[cfg(feature = "report_wasi_calls")]
    let start = ic_instruction_counter();

    let result = wasi::ERRNO_SUCCESS.raw() as i32;

    #[cfg(feature = "report_wasi_calls")]
    debug_instructions!(
        "__fx_custom_fd_prestat_get fd={}",
        result,
        start,
        "fd={fd:?} -> res={res:?}"
    );

    result
}

#[no_mangle]
#[inline(never)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __fx_custom_fd_prestat_dir_name(fd: i32, path: *mut u8, max_len: i32) -> i32 {
    #[cfg(feature = "report_wasi_calls")]
    let start = ic_instruction_counter();

    let max_len = max_len as wasi::Size;

    let result = wasi::ERRNO_SUCCESS.raw() as i32;

    #[cfg(feature = "report_wasi_calls")]
    debug_instructions!("__fx_custom_fd_prestat_dir_name", result, start, "fd={fd:?}");

    result
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __fx_custom_fd_advise(fd: i32, offset: i64, len: i64, advice: i32) -> i32 {
    #[cfg(feature = "report_wasi_calls")]
    let start = ic_instruction_counter();

    prevent_elimination(&[offset as i32, len as i32]);

    if advice as u32 > 5 {
        return wasi::ERRNO_INVAL.raw() as i32;
    }

    let mut is_badf = false;

    let result = if is_badf {
        wasi::ERRNO_BADF.raw() as i32
    } else {
        wasi::ERRNO_SUCCESS.raw() as i32
    };

    #[cfg(feature = "report_wasi_calls")]
    debug_instructions!(
        "__fx_custom_fd_advise",
        result,
        start,
        "fd={fd:?} offset={offset:?} len={len:?} advice={advice:?}"
    );

    result
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __fx_custom_fd_allocate(fd: i32, offset: i64, len: i64) -> i32 {
    #[cfg(feature = "report_wasi_calls")]
    let start = ic_instruction_counter();

    prevent_elimination(&[offset as i32, len as i32]);

    let mut result = wasi::ERRNO_SUCCESS.raw() as i32;

    #[cfg(feature = "report_wasi_calls")]
    debug_instructions!(
        "__fx_custom_fd_allocate",
        result,
        start,
        "fd={fd:?} offset={offset:?} len={len:?}"
    );

    result
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __fx_custom_fd_datasync(fd: i32) -> i32 {
    #[cfg(feature = "report_wasi_calls")]
    let start = ic_instruction_counter();

    let mut result = wasi::ERRNO_SUCCESS.raw() as i32;

    #[cfg(feature = "report_wasi_calls")]
    debug_instructions!("__fx_custom_fd_datasync", result, start, "fd={fd:?}");

    result
}

#[no_mangle]
#[inline(never)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __fx_custom_fd_fdstat_get(fd: i32, ret_fdstat: *mut wasi::Fdstat) -> i32 {
    #[cfg(feature = "report_wasi_calls")]
    let start = ic_instruction_counter();

    let result = wasi::ERRNO_SUCCESS.raw() as i32;

    #[cfg(feature = "report_wasi_calls")]
    debug_instructions!("__fx_custom_fd_fdstat_get", result, start, "fd={fd:?}");

    result
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __fx_custom_fd_fdstat_set_flags(fd: i32, new_flags: i32) -> i32 {
    #[cfg(feature = "report_wasi_calls")]
    let start = ic_instruction_counter();

    let result = wasi::ERRNO_SUCCESS.raw() as i32;

    #[cfg(feature = "report_wasi_calls")]
    debug_instructions!(
        "__fx_custom_fd_fdstat_set_flags",
        result,
        start,
        "fd={fd:?} new_flags={new_flags:?}"
    );

    result
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __fx_custom_fd_fdstat_set_rights(fd: i32, rights_base: i64, rights_inheriting: i64) -> i32 {
    #[cfg(feature = "report_wasi_calls")]
    let start = ic_instruction_counter();

    let result = wasi::ERRNO_SUCCESS.raw() as i32;

    #[cfg(feature = "report_wasi_calls")]
    debug_instructions!(
        "__fx_custom_fd_fdstat_set_rights",
        result,
        start,
        "fd={fd:?} rights_base={rights_base:?} rights_inheriting={rights_inheriting:?}"
    );

    result
}

#[no_mangle]
pub extern "C" fn __fx_custom_fd_filestat_set_size(fd: i32, size: i64) -> i32 {
    #[cfg(feature = "report_wasi_calls")]
    let start = ic_instruction_counter();

    prevent_elimination(&[size as i32]);

    let result = wasi::ERRNO_SUCCESS.raw() as i32;

    #[cfg(feature = "report_wasi_calls")]
    debug_instructions!(
        "__fx_custom_fd_filestat_set_size",
        result,
        start,
        "fd={fd:?} size={size:?}"
    );

    result
}

#[no_mangle]
pub extern "C" fn __fx_custom_fd_filestat_set_times(fd: i32, atim: i64, mtim: i64, fst_flags: i32) -> i32 {
    #[cfg(feature = "report_wasi_calls")]
    let start = ic_instruction_counter();

    let fst_flags = fst_flags as wasi::Fstflags;

    let result = wasi::ERRNO_SUCCESS.raw() as i32;

    #[cfg(feature = "report_wasi_calls")]
    debug_instructions!(
        "__fx_custom_fd_filestat_set_times",
        result,
        start,
        "fd={fd:?} atim={atim:?} mtim={mtim:?} fst_flags={fst_flags:?}"
    );

    result
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __fx_custom_fd_readdir(
    fd: i32,
    bytes: *mut u8,
    bytes_len: i32,
    cookie: i64,
    res: *mut wasi::Size,
) -> i32 {
    #[cfg(feature = "report_wasi_calls")]
    let start = ic_instruction_counter();

    let result = wasi::ERRNO_SUCCESS.raw() as i32;

    #[cfg(feature = "report_wasi_calls")]
    {
        let mn = std::cmp::min(std::cmp::min(bytes_len as usize, unsafe { *res } as usize), 50);
        let buf = unsafe { std::slice::from_raw_parts_mut(bytes, mn) };

        let parms = format!(
            "fd={fd:?} cookie={cookie:?} buf={buf:?}... bytes_len={bytes_len:?} res={:?}",
            unsafe { *res }
        );
        debug_instructions!("__fx_custom_fd_readdir", result, start, "{parms}");
    }

    result
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __fx_custom_fd_renumber(fd_from: i32, fd_to: i32) -> i32 {
    #[cfg(feature = "report_wasi_calls")]
    let start = ic_instruction_counter();

    let result = wasi::ERRNO_SUCCESS.raw() as i32;

    #[cfg(feature = "report_wasi_calls")]
    debug_instructions!(
        "__fx_custom_fd_renumber",
        result,
        start,
        "fd_from={fd_from:?} fd_to={fd_to:?}"
    );

    result
}

#[no_mangle]
#[inline(never)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __fx_custom_random_get(buf: *mut u8, buf_len: wasi::Size) -> i32 {
    #[cfg(feature = "report_wasi_calls")]
    let start = ic_instruction_counter();

    let buf = std::slice::from_raw_parts_mut(buf, buf_len);
    /*RNG.with(|rng| {
        let mut rng = rng.borrow_mut();
        rng.fill_bytes(buf);
    });*/

    let result = wasi::ERRNO_SUCCESS.raw() as i32;

    #[cfg(feature = "report_wasi_calls")]
    debug_instructions!("__fx_custom_random_get", result, start);

    result
}

#[no_mangle]
#[inline(never)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __fx_custom_environ_get(environment: *mut *mut u8, environment_buffer: *mut u8) -> i32 {
    #[cfg(feature = "report_wasi_calls")]
    let start = ic_instruction_counter();

    let result = 0;

    #[cfg(feature = "report_wasi_calls")]
    debug_instructions!("__fx_custom_environ_get", result, start);

    result
}

#[no_mangle]
#[inline(never)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __fx_custom_environ_sizes_get(
    entry_count: *mut wasi::Size,
    buffer_size: *mut wasi::Size,
) -> i32 {
    #[cfg(feature = "report_wasi_calls")]
    let start = ic_instruction_counter();

    /*ENV.with(|env| {
        let env = env.borrow();
        let (count, size) = env.environ_sizes_get();

        *entry_count = count;
        *buffer_size = size;
    });*/

    let result = 0;

    #[cfg(feature = "report_wasi_calls")]
    debug_instructions!("__fx_custom_environ_sizes_get", result, start);

    result
}

macro_rules! PA_HND_TYPE {
    ($n:expr) => {
        ($n & 0xFF)
    };
}

macro_rules! PA_HND_ARG {
    ($n:expr) => {
        ($n >> 16) & 0xFFFF
    };
}

unsafe fn processargs_read(
    bootstrap: fx_handle_t,
    buffer: *mut u8,
    nbytes: usize,
    handles: *mut u32,
    nhandles: u32,
    pargs: &mut fx_proc_args_t,
    handle_info: &mut *mut u32,
) -> fx_status_t {
    if nbytes < std::mem::size_of::<fx_proc_args_t>() {
        return FX_ERR_INVALID_ARGS;
    }

    if (buffer as usize) % std::mem::align_of::<fx_proc_args_t>() != 0 {
        panic!("{:?} {:?}", buffer as usize, std::mem::align_of::<fx_proc_args_t>());
        return FX_ERR_INVALID_ARGS;
    }

    let mut got_bytes = 0;
    let mut got_handles = 0;
    let status = fx_channel_read(
        bootstrap,
        0,
        buffer,
        handles,
        nbytes as u32,
        nhandles,
        &mut got_bytes,
        &mut got_handles,
    );
    if status != FX_OK {
        panic!("read");
        return status;
    }

    if got_bytes != (nbytes as u32) || got_handles != nhandles {
        panic!("one");
        return FX_ERR_INVALID_ARGS;
    }

    let pa = buffer as *mut fx_proc_args_t;

    //panic!("{:?}", *pa);

    if (*pa).protocol != FX_PROCARGS_PROTOCOL || (*pa).version != FX_PROCARGS_VERSION {
        panic!("two");
        return FX_ERR_INVALID_ARGS;
    }

    if (*pa).handle_info_off < std::mem::size_of::<fx_proc_args_t>() as u32
        || (*pa).handle_info_off % std::mem::align_of::<u32>() as u32 != 0
        || (*pa).handle_info_off > (nbytes as u32)
        || ((nbytes as u32) - (*pa).handle_info_off) / (std::mem::size_of::<u32>() as u32) < nhandles
    {
        panic!("three");
        return FX_ERR_INVALID_ARGS;
    }

    /*if pa.args_num > 0 && (pa.args_off < sizeof(*pa) || pa.args_off > nbytes || (nbytes - pa.args_off) < pa.args_num) {
        return FX_ERR_INVALID_ARGS;
    }

    if (pa.environ_num > 0 && (pa.environ_off < sizeof(*pa) || pa.environ_off > nbytes ||
                              (nbytes - pa.environ_off) < pa.environ_num)) {
        return FX_ERR_INVALID_ARGS;
    }*/

    *pargs = *pa;
    *handle_info = buffer.wrapping_add((*pa).handle_info_off as usize) as *mut u32;

    return FX_OK;
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

/// The function to unpack strings from the buffer
/// Safety: We assume the caller ensures buffer points to a valid memory region
unsafe fn unpack_strings(buffer: *mut u8, nbytes: u32, result: &mut Vec<CString>, off: u32, num: u32) -> fx_status_t {
    let mut p = (buffer as *mut i8).add(off as usize);

    for _ in 0..num {
        // Find the length of the string
        let start = p;
        let mut length = 0;

        while *p != 0 {
            // Check bounds to ensure we're not reading beyond the buffer
            if p >= (buffer as *mut i8).add(nbytes as usize) {
                return FX_ERR_INVALID_ARGS;
            }
            p = p.add(1);
            length += 1;
        }

        // Move past the null terminator
        p = p.add(1);
        //length += 1;

        // Convert the slice to CString
        let slice = std::slice::from_raw_parts(start as *const u8, length as usize);
        
        match CString::new(slice) {
            Ok(cstring) => {
                println!("{:?}", cstring.as_bytes_with_nul());
                result.push(cstring)},
            Err(_) => return FX_ERR_INVALID_ARGS, // Handle CString creation failure
        }
    }

    // Null-terminate the result vector
    //result.push(CString::new("").unwrap());

    FX_OK
}

fn processargs_strings(
    pa: &mut fx_proc_args_t,
    buffer: *mut u8,
    nbytes: u32,
    argv: &mut Vec<CString>,
    envp: &mut Vec<CString>,
    names: &mut Vec<CString>,
) -> fx_status_t {
    let pa = buffer as *mut fx_proc_args_t;

    let mut status = FX_OK;

    status = unsafe { unpack_strings(buffer, nbytes, argv, (*pa).args_off, (*pa).args_num) };

    if status == FX_OK {
        status = unsafe { unpack_strings(buffer, nbytes, envp, (*pa).environ_off, (*pa).environ_num) };
    }

    if status == FX_OK {
        status = unsafe { unpack_strings(buffer, nbytes, names, (*pa).names_off, (*pa).names_num) };
    }

    status
}

unsafe fn processargs_extract_handles(
    nhandles: u32,
    handles: &mut [fx_handle_t],
    handle_info: &mut [u32],
    process_self: *mut fx_handle_t,
    job_default: *mut fx_handle_t,
    utc_reference: *mut fx_handle_t,
) {
    // Find the handles we're interested in among what we were given.
    for i in 0..(nhandles as usize) {
        println!("type={:?}", PA_HND_TYPE!(handle_info[i]));
        println!("arg={:?}", PA_HND_ARG!(handle_info[i]));

        match PA_HND_TYPE!(handle_info[i]) {
            PA_PROC_SELF => {
                *process_self = handles[i];
                handles[i] = FX_HANDLE_INVALID;
                handle_info[i] = 0;
                break;
            }
            PA_JOB_DEFAULT => {
                *process_self = handles[i];
                handles[i] = FX_HANDLE_INVALID;
                handle_info[i] = 0;
                break;
            }
            _ => panic!("unsupported"),
        }
        //switch (PA_HND_TYPE(handle_info[i])) {
        //  case PA_PROC_SELF:
    }
}

#[derive(Debug)]
struct StartParams {
    procargs: fx_proc_args_t,
    nbytes: u32,
    nhandles: u32,
    handle_info: *mut u32,
    buffer: *mut u8,
    handles: *mut fx_handle_t,
}

#[no_mangle]
pub extern "C" fn __fx_init(bootstrap: fx_handle_t) {
    let mut p = StartParams {
        nbytes: 0,
        nhandles: 0,
        buffer: std::ptr::null_mut(),
        handles: std::ptr::null_mut(),
        handle_info: std::ptr::null_mut(),
        procargs: Default::default(),
    };

    // extract process startup information from channel in arg
    //struct start_params p = {.main = main, .utc_reference = ZX_HANDLE_INVALID};
    let status = processargs_message_size(bootstrap, &mut p.nbytes, &mut p.nhandles);

    // TODO(44088): Right now, we _always_ expect to receive at least some
    // handles and some bytes in the initial startup message.  Make sure that we
    // have both so that we do not accidentally end up declaring a 0-length VLA
    // on the stack (which is UDB in C11).  See the bug referenced in the TODO,
    // however.  We do not currently formally state that this is a requirement
    // for starting a process, nor do we declare a maximum number of handles
    // which can be sent during startup.  Restructuring and formalizing the
    // process-args startup protocol could help with this situation.
    if (status == FX_OK) && p.nbytes != 0 && p.nhandles != 0 {
        let mut buffer: Vec<u8> = vec![0; p.nbytes as usize];
        let mut handles: Vec<fx_handle_t> = vec![fx_handle_t::default(); p.nhandles as usize]; // Initialize handles
        p.buffer = buffer.as_mut_ptr();
        p.handles = handles.as_mut_ptr();

        let status = unsafe {
            processargs_read(
                bootstrap,
                buffer.as_mut_ptr(),
                p.nbytes as usize,
                handles.as_mut_ptr(),
                p.nhandles,
                &mut p.procargs,
                &mut p.handle_info,
            )
        };

        unsafe {
            let mut handle_infos = std::slice::from_raw_parts_mut(p.handle_info, p.nhandles as usize);
            println!("handle_infos={:?}", handle_infos);
            processargs_extract_handles(p.nhandles, &mut handles, handle_infos, &mut 0, &mut 0, &mut 0);
        }

        println!("status={} handle_values={:?}", status, handles);

        if status != FX_OK {
            panic!("status={} invalid processargs read", status);
        }

        let argc = p.procargs.args_num as usize;
        let envc = p.procargs.environ_num as usize;
        let namec = p.procargs.names_num as usize;

        // Allocate vectors for argv, environ, and names
        //let mut argv: Vec<CString> = Vec::with_capacity(argc);
        let mut environ: Vec<CString> = Vec::with_capacity(envc);
        let mut names: Vec<CString> = Vec::with_capacity(namec);

        ARGS.with(|args| {
            let mut args = args.borrow_mut();
            args.data_values.reserve(argc);
            args.data_size = 26; // TODO: remove hardcode

            //env.set_environment(env_pairs);

            // Unsafe block to call external C function and handle raw pointers
            let status = unsafe {
                processargs_strings(
                    &mut p.procargs,
                    buffer.as_mut_ptr(),
                    p.nbytes,
                    &mut args.data_values,
                    &mut environ,
                    &mut names,
                )
            };

            println!(
                "status={} argv={:?} environ={:?} names={:?}",
                status, args.data_values, environ, names
            );
        });

        // Initialize vectors with null pointers
        //argv.resize(argc + 1, unsafe { (std::ptr::null_mut()) });
        //environ.resize(envc + 1, std::ptr::null_mut());
        //names.resize(namec + 1, std::ptr::null_mut());

        // Fill argv and environ with example data (CStrings for simulation)
        //let example_args = vec!["program_name", "arg1", "arg2"];
        //let example_envs = vec!["ENV_VAR1=value1", "ENV_VAR2=value2"];

        // Set argv with pointers to example argument CStrings
        //for (i, arg) in example_args.iter().enumerate() {
        //    let cstr = CString::new(*arg).unwrap();
        //    let ptr = cstr.into_raw();
        //    unsafe { *argv.add(i) = ptr as *mut c_void };
        //}
    }

    println!("status={} info={:?}", status, p)
}

#[no_mangle]
pub extern "C" fn __fx_custom_proc_exit(code: i32) -> ! {
    panic!("WASI proc_exit called with code: {code}");
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn __fx_custom_args_get(arg_entries: *mut *mut u8, arg_buffer: *mut u8) -> i32 {
    #[cfg(feature = "report_wasi_calls")]
    debug_instructions!("__fx_custom_args_get -> 0");

    let result = ARGS.with(|args| {
        let args = args.borrow();
        args.arg_get(arg_entries, arg_buffer)
    });

    let result = result.raw() as i32;

    result
}

#[no_mangle]
#[inline(never)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __fx_custom_args_sizes_get(entry_count: *mut wasi::Size, buffer_size: *mut wasi::Size) -> i32 {
    #[cfg(feature = "report_wasi_calls")]
    let start = ic_instruction_counter();

    ARGS.with(|args| {
        let args = args.borrow();
        let (count, size) = args.arg_sizes_get();

        *entry_count = count;
        *buffer_size = size;
    });

    let result = wasi::ERRNO_SUCCESS.raw() as i32;

    #[cfg(feature = "report_wasi_calls")]
    debug_instructions!("__fx_custom_args_sizes_get", result, start);

    result
}

#[no_mangle]
#[inline(never)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __fx_custom_clock_res_get(id: i32, result: *mut u64) -> i32 {
    prevent_elimination(&[id]);

    #[cfg(feature = "report_wasi_calls")]
    debug_instructions!("__fx_custom_clock_res_get -> 0");

    *result = 1_000_000_000; // 1 second.
    wasi::ERRNO_SUCCESS.raw() as i32
}

#[no_mangle]
#[inline(never)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __fx_custom_clock_time_get(id: i32, precision: i64, time: *mut u64) -> i32 {
    prevent_elimination(&[id, precision as i32]);

    #[cfg(feature = "report_wasi_calls")]
    let start = ic_instruction_counter();

    *time = ic_time();
    let result = wasi::ERRNO_SUCCESS.raw() as i32;

    #[cfg(feature = "report_wasi_calls")]
    debug_instructions!("__fx_custom_clock_time_get", result, start);

    result
}

#[no_mangle]
#[inline(never)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __fx_custom_path_create_directory(parent_fd: i32, path: *const u8, path_len: i32) -> i32 {
    #[cfg(feature = "report_wasi_calls")]
    let start = ic_instruction_counter();

    let dir_name = get_file_name(path, path_len as wasi::Size);

    let result = 0;

    #[cfg(feature = "report_wasi_calls")]
    debug_instructions!(
        "__fx_custom_path_create_directory",
        result,
        start,
        "parent_fd={parent_fd:?} dir_name={dir_name:?}"
    );

    result
}

#[no_mangle]
#[inline(never)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn __fx_custom_path_filestat_get(
    parent_fd: i32,
    simlink_flags: i32,
    path: *const u8,
    path_len: i32,
    result: *mut wasi::Filestat,
) -> i32 {
    #[cfg(feature = "report_wasi_calls")]
    let start = ic_instruction_counter();
    let file_name = get_file_name(path, path_len as wasi::Size);

    prevent_elimination(&[simlink_flags]);

    let result = 0;

    #[cfg(feature = "report_wasi_calls")]
    debug_instructions!(
        "__fx_custom_path_filestat_get",
        result,
        start,
        "parent_fd={parent_fd:?} file_name={file_name:?}"
    );

    result
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __fx_custom_path_filestat_set_times(
    parent_fd: i32,
    flags: i32,
    path: *const u8,
    path_len: i32,
    atim: i64,
    mtim: i64,
    fst_flags: i32,
) -> i32 {
    #[cfg(feature = "report_wasi_calls")]
    let start = ic_instruction_counter();
    prevent_elimination(&[flags]);
    let file_name = get_file_name(path, path_len as wasi::Size);

    let result = 0;

    #[cfg(feature = "report_wasi_calls")]
    debug_instructions!(
        "__fx_custom_path_filestat_set_times",
        result,
        start,
        "parent_fd={parent_fd:?} file_name={file_name:?} atim={atim:?} mtim={mtim:?}"
    );

    result
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __fx_custom_path_link(
    old_fd: i32,
    old_flags: i32,
    old_path: *const u8,
    old_path_len: i32,
    new_fd: i32,
    new_path: *const u8,
    new_path_len: i32,
) -> i32 {
    #[cfg(feature = "report_wasi_calls")]
    let start = ic_instruction_counter();
    prevent_elimination(&[old_flags]);
    let old_path = get_file_name(old_path, old_path_len as wasi::Size);
    let new_path = get_file_name(new_path, new_path_len as wasi::Size);

    let result = 0;

    #[cfg(feature = "report_wasi_calls")]
    debug_instructions!(
        "__fx_custom_path_link",
        result,
        start,
        "old_parent_fd={old_fd:?} old_path={old_path:?} <- new_parent_fd={new_fd:?} new_path={new_path:?}"
    );

    result
}

#[no_mangle]
#[inline(never)]
#[cfg(not(feature = "skip_unimplemented_functions"))]
pub extern "C" fn __fx_custom_path_readlink(
    fd: i32,
    path: *const u8,
    path_len: i32,
    buf: i32,
    buf_len: i32,
    rp0: i32,
) -> i32 {
    prevent_elimination(&[fd, path as i32, path_len, buf, buf_len, rp0]);
    unimplemented!("WASI path_readlink is not implemented");
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __fx_custom_path_remove_directory(parent_fd: i32, path: *const u8, path_len: i32) -> i32 {
    #[cfg(feature = "report_wasi_calls")]
    let start = ic_instruction_counter();
    let file_name = get_file_name(path, path_len as wasi::Size);

    let result = 0;

    #[cfg(feature = "report_wasi_calls")]
    debug_instructions!(
        "__fx_custom_path_remove_directory",
        result,
        start,
        "parent_fd={parent_fd:?} file_name={file_name:?}"
    );

    result
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __fx_custom_path_rename(
    old_fd: i32,
    old_path: *const u8,
    old_path_len: i32,
    new_fd: i32,
    new_path: *const u8,
    new_path_len: i32,
) -> i32 {
    #[cfg(feature = "report_wasi_calls")]
    let start = ic_instruction_counter();

    let result = 0;

    #[cfg(feature = "report_wasi_calls")]
    debug_instructions!(
        "__fx_custom_path_rename",
        result,
        start,
        "old_parent_fd={old_fd:?} old_path={old_path:?} -> new_parent_fd={new_fd:?} new_path={new_path:?}"
    );

    result
}

#[no_mangle]
#[inline(never)]
#[cfg(not(feature = "skip_unimplemented_functions"))]
pub extern "C" fn __fx_custom_path_symlink(
    old_path: i32,
    old_path_len: i32,
    fd: i32,
    new_path: i32,
    new_path_len: i32,
) -> i32 {
    prevent_elimination(&[old_path, old_path_len, fd, new_path, new_path_len]);
    unimplemented!("WASI path_symlink is not implemented");
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __fx_custom_path_unlink_file(parent_fd: i32, path: *const u8, path_len: i32) -> i32 {
    #[cfg(feature = "report_wasi_calls")]
    let start = ic_instruction_counter();

    let file_name = get_file_name(path, path_len as wasi::Size);

    let result = 0;

    #[cfg(feature = "report_wasi_calls")]
    debug_instructions!(
        "__fx_custom_path_unlink",
        result,
        start,
        "parent_fd={parent_fd:?} file_name={file_name:?}"
    );

    result
}

#[no_mangle]
#[inline(never)]
#[cfg(not(feature = "skip_unimplemented_functions"))]
pub extern "C" fn __fx_custom_poll_oneoff(
    in_: *const wasi::Subscription,
    out: *mut wasi::Event,
    nsubscriptions: i32,
    rp0: i32,
) -> i32 {
    prevent_elimination(&[in_ as i32, out as i32, nsubscriptions, rp0]);
    unimplemented!("WASI poll_oneoff is not implemented");
}

#[no_mangle]
#[inline(never)]
#[cfg(not(feature = "skip_unimplemented_functions"))]
pub extern "C" fn __fx_custom_proc_raise(sig: i32) -> i32 {
    prevent_elimination(&[sig]);
    unimplemented!("WASI proc_raise is not implemented");
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn __fx_custom_sched_yield() -> i32 {
    // No-op.
    0
}

#[no_mangle]
#[inline(never)]
#[cfg(not(feature = "skip_unimplemented_functions"))]
pub extern "C" fn __fx_custom_sock_accept(arg0: i32, arg1: i32, arg2: i32) -> i32 {
    prevent_elimination(&[arg0, arg1, arg2]);
    unimplemented!("WASI sock_accept is not supported");
}

#[no_mangle]
#[inline(never)]
#[cfg(not(feature = "skip_unimplemented_functions"))]
pub extern "C" fn __fx_custom_sock_recv(arg0: i32, arg1: i32, arg2: i32, arg3: i32, arg4: i32, arg5: i32) -> i32 {
    prevent_elimination(&[arg0, arg1, arg2, arg3, arg4, arg5]);
    unimplemented!("WASI sock_recv is not supported");
}

#[no_mangle]
#[inline(never)]
#[cfg(not(feature = "skip_unimplemented_functions"))]
pub extern "C" fn __fx_custom_sock_send(arg0: i32, arg1: i32, arg2: i32, arg3: i32, arg4: i32) -> i32 {
    prevent_elimination(&[arg0, arg1, arg2, arg3, arg4]);
    unimplemented!("WASI sock_send is not supported");
}

#[no_mangle]
#[inline(never)]
#[cfg(not(feature = "skip_unimplemented_functions"))]
pub extern "C" fn __fx_custom_sock_shutdown(arg0: i32, arg1: i32) -> i32 {
    prevent_elimination(&[arg0, arg1]);
    unimplemented!("WASI sock_shutdown is not supported");
}

thread_local! {
    static COUNTER: RefCell<i32> = const {RefCell::new(0)};
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "fiber")]
extern "C" {
    pub fn fx_debug(data: *mut u8, len: usize);
}
#[cfg(not(all(target_arch = "wasm32")))]
fn fx_debug(data: *mut u8, len: usize) {
}

fn prevent_elimination(args: &[i32]) {
    COUNTER.with(|var| {
        if *var.borrow() == -1 {
            let str = format!("args: {args:?}");
            ic_print(str.as_str())
        }
    });
}

#[no_mangle]
pub fn init_seed(seed: &[u8]) {
    unsafe {
        raw_init_seed(seed.as_ptr(), seed.len());
    }
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn raw_init_seed(seed: *const u8, len: usize) {
    if seed.is_null() || len == 0 {
        return;
    }

    let len = usize::min(len, 32);

    let mut seed_buf: [u8; 32] = [0u8; 32];
    unsafe { std::ptr::copy_nonoverlapping(seed, seed_buf.as_mut_ptr(), len) }

    /*RNG.with(|rng| {
        let mut rng = rng.borrow_mut();
        *rng = rand::rngs::StdRng::from_seed(seed_buf);
    });*/
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
#[cfg(not(tarpaulin_include))]
pub unsafe extern "C" fn raw_init() {
    /*FS.with(|fs| {
        let mut fs = fs.borrow_mut();

        if fs.get_storage_version() == 0 {
            *fs = if cfg!(feature = "transient") {
                FileSystem::new(Box::new(TransientStorage::new())).unwrap()
            } else {
                FileSystem::new(Box::new(StableStorage::new(DefaultMemoryImpl::default()))).unwrap()
            }
        }
    });*/

    // raw_init_seed(seed, len);

    COUNTER.with(|var| {
        if *var.borrow() == -1 {
            // dummy calls to trick the linker not to throw away the functions
            unsafe {
                use std::ptr::{null, null_mut};
                __fx_custom_fd_write(0, null::<wasi::Ciovec>(), 0, null_mut::<wasi::Size>());
                __fx_custom_fd_read(0, null::<wasi::Ciovec>(), 0, null_mut::<wasi::Size>());
                __fx_custom_fd_close(0);

                __fx_custom_fd_prestat_get(0, null_mut::<wasi::Prestat>());
                __fx_custom_fd_prestat_dir_name(0, null_mut::<u8>(), 0);

                __fx_custom_path_open(0, 0, null::<u8>(), 0, 0, 0, 0, 0, null_mut::<i32>());
                __fx_custom_random_get(null_mut::<u8>(), 0);

                __fx_custom_environ_get(null_mut::<*mut u8>(), null_mut::<u8>());
                __fx_custom_environ_sizes_get(null_mut::<wasi::Size>(), null_mut::<wasi::Size>());

                __fx_custom_args_get(null_mut::<*mut u8>(), null_mut::<u8>());
                __fx_custom_args_sizes_get(null_mut::<wasi::Size>(), null_mut::<wasi::Size>());
                __fx_custom_clock_res_get(0, null_mut::<u64>());
                __fx_custom_clock_time_get(0, 0, null_mut::<u64>());

                __fx_custom_fd_advise(0, 0, 0, 0);
                __fx_custom_fd_allocate(0, 0, 0);
                __fx_custom_fd_datasync(0);
                __fx_custom_fd_fdstat_get(0, null_mut::<wasi::Fdstat>());
                __fx_custom_fd_fdstat_set_flags(0, 0);
                __fx_custom_fd_fdstat_set_rights(0, 0, 0);
                __fx_custom_fd_filestat_get(0, null_mut::<wasi::Filestat>());
                __fx_custom_fd_filestat_set_size(0, 0);
                __fx_custom_fd_filestat_set_times(0, 0, 0, 0);
                __fx_custom_fd_pread(0, null::<wasi::Ciovec>(), 0, 0, null_mut::<wasi::Size>());
                __fx_custom_fd_pwrite(0, null::<wasi::Ciovec>(), 0, 0, null_mut::<wasi::Size>());
                __fx_custom_fd_readdir(0, null_mut::<u8>(), 0, 0, null_mut::<wasi::Size>());
                __fx_custom_fd_renumber(0, 0);
                __fx_custom_fd_seek(0, 0, 0, null_mut::<wasi::Filesize>());
                __fx_custom_fd_sync(0);
                __fx_custom_fd_tell(0, null_mut::<wasi::Filesize>());
                __fx_custom_path_create_directory(0, null::<u8>(), 0);
                __fx_custom_path_filestat_get(0, 0, null::<u8>(), 0, null_mut::<wasi::Filestat>());
                __fx_custom_path_filestat_set_times(0, 0, null::<u8>(), 0, 0, 0, 0);
                __fx_custom_path_link(0, 0, null::<u8>(), 0, 0, null::<u8>(), 0);

                #[cfg(not(feature = "skip_unimplemented_functions"))]
                __fx_custom_path_readlink(0, null::<u8>(), 0, 0, 0, 0);

                __fx_custom_path_remove_directory(0, null::<u8>(), 0);
                __fx_custom_path_rename(0, null::<u8>(), 0, 0, null::<u8>(), 0);

                #[cfg(not(feature = "skip_unimplemented_functions"))]
                __fx_custom_path_symlink(0, 0, 0, 0, 0);

                __fx_custom_path_unlink_file(0, null::<u8>(), 0);

                #[cfg(not(feature = "skip_unimplemented_functions"))]
                __fx_custom_poll_oneoff(null::<wasi::Subscription>(), null_mut::<wasi::Event>(), 0, 0);
                #[cfg(not(feature = "skip_unimplemented_functions"))]
                __fx_custom_proc_raise(0);
                __fx_custom_sched_yield();

                #[cfg(not(feature = "skip_unimplemented_functions"))]
                __fx_custom_sock_accept(0, 0, 0);
                #[cfg(not(feature = "skip_unimplemented_functions"))]
                __fx_custom_sock_recv(0, 0, 0, 0, 0, 0);
                #[cfg(not(feature = "skip_unimplemented_functions"))]
                __fx_custom_sock_send(0, 0, 0, 0, 0);
                #[cfg(not(feature = "skip_unimplemented_functions"))]
                __fx_custom_sock_shutdown(0, 0);

                __fx_custom_proc_exit(0);
            }
        }
    })
}

// the init function ensures the module is not thrown away by the linker
// seed       -  The seed of the random numbers, up to 32 byte array can be used.
// env_pairs  -  The pre-defined environment variables.
//
// Example:
// init(&[12,3,54,1], &[("PATH", "/usr/bin"), ("UID", "1028"), ("HOME", "/home/user")]);
#[allow(clippy::missing_safety_doc)]
pub fn init() {
    /*ENV.with(|env| {
        let mut env = env.borrow_mut();
        env.set_environment(env_pairs);
    });*/

    unsafe {
        raw_init();
    }
}

/*#[allow(clippy::missing_safety_doc)]
pub fn init_with_memory<M: Memory + 'static>(seed: &[u8], env_pairs: &[(&str, &str)], memory: M) {
    /*FS.with(|fs| {
        let mut fs = fs.borrow_mut();

        *fs = FileSystem::new(Box::new(StableStorage::new(memory))).unwrap();
    });*/

    init(seed, env_pairs);
}*/

/*#[allow(clippy::missing_safety_doc)]
pub fn init_with_memory_manager<M: Memory + 'static>(
    seed: &[u8],
    env_pairs: &[(&str, &str)],
    memory_manager: &MemoryManager<M>,
    memory_index_range: Range<u8>,
) {
    /*FS.with(|fs| {
        let mut fs = fs.borrow_mut();

        *fs = FileSystem::new(Box::new(StableStorage::new_with_memory_manager(
            memory_manager,
            memory_index_range,
        )))
        .unwrap();
    });*/

    init(seed, env_pairs);
}*/

#[cfg(test)]
mod lib_test;

#[cfg(test)]
mod integration_tests;
