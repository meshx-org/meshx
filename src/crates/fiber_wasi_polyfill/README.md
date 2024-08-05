# The Fiber WASI Polyfill library

![Tests](https://github.com/wasm-forge/ic-wasi-polyfill/actions/workflows/rust.yml/badge.svg?event=push)
[![Coverage](https://codecov.io/gh/wasm-forge/ic-wasi-polyfill/branch/main/graph/badge.svg)](https://codecov.io/gh/wasm-forge/ic-wasi-polyfill/branch/main/)

The project provides polyfill implementation of *wasi_unstable* and *wasi_snapshot_preview1* functions using Fiber Syscall API.


## Usage

The intended use is to add this library as a dependency to your rust project. And then run `wasi2fx` on the produced Wasm binary.

In your project you would need to call the `init` function. It makes sure the linker does not remove the functions and can be used to initialize the random seed and define some environment variables.

Example:
```rust
    init(&[12,3,54,1], &[("PATH", "/usr/bin"), ("UID", "1028"), ("HOME", "/home/user")]);
```


## Supported WASI functions (wasi_unstable, wasi_snapshot_preview1)


| Status           | Description                                                  |
| ---------------- | ------------------------------------------------------------ |
| Supported        | Function is fully supported.                                 |
| Not implemented            | Empty implementation that does nothing but can be called without issues. |
| Not implemented  | Function is not yet implemented - calling it causes the application to panic. |
| Not supported    | Function is not planned to be implemented - calling it causes the application to panic. |


| WASI function               | Status                | 
| --------------------------- | ----------------------|
| `args_get`                  | Not implemented       |
| `args_sizes_get`            | Not implemented       |
| `clock_res_get`             | Not implemented       |
| `clock_time_get`            | Not implemented       |
| `environ_get`               | Not implemented       |
| `environ_sizes_get`         | Not implemented       |
| `fd_advise`                 | Not implemented       |
| `fd_allocate`               | Not implemented       |
| `fd_close`                  | Not implemented       |
| `fd_datasync`               | Not implemented       |
| `fd_fdstat_get`             | Not implemented       |
| `fd_fdstat_set_flags`       | Not implemented       |
| `fd_fdstat_set_rights`      | Not implemented       |
| `fd_filestat_get`           | Not implemented       |
| `fd_filestat_set_size`      | Not implemented       |
| `fd_filestat_set_times`     | Not implemented       |
| `fd_pread`                  | Not implemented       |
| `fd_prestat_dir_name`       | Not implemented       |
| `fd_prestat_get`            | Not implemented       |
| `fd_pwrite`                 | Not implemented       |
| `fd_read`                   | Not implemented       |
| `fd_readdir`                | Not implemented       |
| `fd_renumber`               | Not implemented       |
| `fd_seek`                   | Not implemented       |
| `fd_sync`                   | Not implemented       |
| `fd_tell`                   | Not implemented       |
| `fd_write`                  | Not implemented       |
| `path_create_directory`     | Not implemented       |
| `path_filestat_get`         | Not implemented       |
| `path_filestat_set_times`   | Not implemented       |
| `path_link`                 | Supported<sup>1</sup> |
| `path_open`                 | Supported<sup>1</sup> |
| `path_readlink`             | Not implemented       |
| `path_remove_directory`     | Not implemented       |
| `path_rename`               | Not implemented       |
| `path_symlink`              | Not implemented       |
| `path_unlink_file`          | Not implemented       |
| `poll_oneoff`               | Not implemented       |
| `proc_exit`                 | Not implemented       |
| `proc_raise`                | Not implemented       |
| `random_get`                | Supported<sup>2</sup> |
| `sched_yield`               | Not implemented       |
| `sock_accept`               | Not supported         |
| `sock_recv`                 | Not supported         |
| `sock_send`                 | Not supported         |
| `sock_shutdown`             | Not supported         |

*<sup>1</sup>* - Currently symlinks are not supported by the file system, this affects a few `path_` functions, the `flags` ("follow symlink") parameter is currently ignored.

*<sup>2</sup>* - The `random_get` function utilizes a synchronous pseudo-random number generator.


## Additional library functions


| Function                                          |  Description                  | 
| ------------------------------------------------- | ----------------------------- |
| `init(seed: &[u8], env_pairs: &[(&str, &str)])`   | Initialization call.          |
| `raw_init(seed: *const u8, len: usize)`           | Similar to `init`, but has simpler parameters for calling from C or C++. |
| `init_seed(seed: &[u8])`                          | Convenience method to explicitly re-initialize the random seed. |
| `raw_init_seed(seed: *const u8, len: usize)`      | Similar to `init_seed`, but has simpler parameters for calling from C or C++. |
| `init_with_memory(seed: &[u8], env_pairs: &[(&str, &str)]), memory: Memory)`    | Initialization on top of custom memory provided by user. |
| `init_with_memory_manager(seed: &[u8], env_pairs: &[(&str, &str)]), memory_manager: &MemoryManager, memory_index_range: Range<u8>)`    | Initialization with the provided memory manager and a range of memory indices to be used by the stable storage. |

## Project features

The polyfill library's behavior can be configured using the following [Cargo features](https://doc.rust-lang.org/cargo/reference/features.html):

* `transient` use the transient file system implementation. This works faster but does not take the advantage of keeping the file system's state in stable memory (and the ability to keep FS state between canister upgrades).
* `report_wasi_calls` outputs statistical information of the called polyfill functions.
* `skip_unimplemented_functions` rather than throw exception on calling the unimplemented function, its implementation will be missing in the compilation. This can be useful if you want to provide custom implementations for those functions.
