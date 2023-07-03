use fiber_rust as fx;
use fiber_rust::HandleBased;

fn bootstrap(channel: fx::Channel) {
    println!("Hello, world from user space!, {:?}", channel);

    let mut buf = fx::MessageBuf::new();
    channel.read(&mut buf);
}

// This is the entry point for the whole show, the very first bit of code
// to run in user mode.
pub fn _start(arg1: fx::sys::fx_handle_t, arg2: fx::sys::fx_handle_t) {
    let handle = unsafe { fx::Handle::from_raw(arg1) };

    println!("before {}", arg2);
    println!("after {}", arg2);

    bootstrap(fx::Channel::from_handle(handle));
}
