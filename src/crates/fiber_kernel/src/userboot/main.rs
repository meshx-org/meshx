use std::vec;
use fiber_rust as fx;
use fiber_rust::HandleBased;
use fx::MessageBuf;

fn bootstrap(channel: fx::Channel) {
    println!("Hello, world from user space!, {:?}", channel);

    let mut buf = fx::MessageBuf::new();
    channel.read(&mut buf);
}

// This is the entry point for the whole show, the very first bit of code
// to run in user mode.
pub fn _start(arg: fx::sys::fx_handle_t) {
    let handle = unsafe { fx::Handle::from_raw(arg) };
    bootstrap(fx::Channel::from_handle(handle));
}
