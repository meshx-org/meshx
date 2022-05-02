mod syscalls;

#[no_mangle]
pub extern "C" fn _start() {
  //unsafe { syscalls::system::get_features(); }

  println!("Hello, world!");

  let _ = 1 + 1;
}

