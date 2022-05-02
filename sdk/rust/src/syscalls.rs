type Handle = u32;

const InvalidHandle: Handle = 0;

pub mod process {
  use crate::syscalls::Handle;

  #[link(wasm_import_module = "meshx::process")]
  extern "C" {
    /// create a new process
    pub fn create();
    pub fn start();
    pub fn exit();
  }
}

pub mod channel {
  use crate::syscalls::Handle;

  #[link(wasm_import_module = "meshx::channel")]
  extern "C" {
    // TODO: pub fn call(handle: Handle); // synchronously send a message and receive a reply
    
    /// create a new channel
    #[link_name = "create"]
    pub fn create(out0: *const Handle, out1: *const Handle);
    
    /// receive a message from a channel
    pub fn read(
      handle: Handle,
      //options: u32,
      bytes: *const u8,
      handles: *const Handle,
      num_bytes: u32,
      num_handles: u32,
      actual_bytes: *const u32,
      actual_handles: *const u32
    );
    
    /// write a message to a channel
    pub fn write(
      handle: Handle,
      //options: u32,
      bytes: *const u8,
      num_bytes: u32,
      handles: *const Handle,
      num_handles: u32
    );
  }
}

// Global system information
pub mod system {
  use crate::syscalls::Handle;

  #[link(wasm_import_module = "meshx::system")]
  extern "C" {
    #[no_mangle]
    #[allow(dead_code)]
    pub fn get_features(); // synchronously send a message and receive a reply   
    
    #[no_mangle]
    #[allow(dead_code)]
    pub fn get_platform_name(); // synchronously send a message and receive a reply   
    
    #[no_mangle]
    #[allow(dead_code)]
    pub fn get_version_string(); // write a message to a channel
  }
}

/*
pub mod process {
  #[link(wasm_import_module = "lunatic::process")]
  extern "C" {
      pub fn create_config(max_memory: u64, max_fuel: u64) -> u64;
      pub fn drop_config(config_id: u64);
      pub fn allow_namespace(config_id: u64, name: *const u8, name_len: usize);
      pub fn preopen_dir(config_id: u64, dir: *const u8, dir_len: usize, id: *mut u64) -> u32;
      pub fn create_environment(config_id: u64, id: *mut u64) -> u32;
      pub fn create_remote_environment(
          config_id: u64,
          node_name: *const u8,
          node_name_len: usize,
          id: *mut u64,
      ) -> u32;
      pub fn drop_environment(env_id: u64);
      pub fn add_module(
          env_id: u64,
          module_data: *const u8,
          module_data_len: usize,
          id: *mut u64,
      ) -> u32;
      pub fn add_this_module(env_id: u64, id: *mut u64) -> u32;
      pub fn drop_module(mod_id: u64);
      pub fn spawn(
          link: i64,
          module_id: u64,
          function: *const u8,
          function_len: usize,
          params: *const u8,
          params_len: usize,
          id: *mut u64,
      ) -> u32;
      pub fn inherit_spawn(
          link: i64,
          function: *const u8,
          function_len: usize,
          params: *const u8,
          params_len: usize,
          id: *mut u64,
      ) -> u32;
      pub fn drop_process(process_id: u64);
      pub fn clone_process(process_id: u64) -> u64;
      pub fn sleep_ms(millis: u64);
      pub fn die_when_link_dies(trap: u32);
      pub fn this() -> u64;
      pub fn id(process_id: u64, uuid: *mut [u8; 16]);
      pub fn this_env() -> u64;
      pub fn link(tag: i64, process_id: u64);
      pub fn unlink(process_id: u64); 
  }
}

*/