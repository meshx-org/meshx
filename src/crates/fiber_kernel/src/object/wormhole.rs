//! async-wormhole allows you to call `.await` async calls across non-async functions, like extern "C" or JIT
//! generated code.
//!
//! ## Motivation
//!
//! Sometimes, when running inside an async environment you need to call into JIT generated code (e.g. wasm)
//! and .await from there. Because the JIT code is not available at compile time, the Rust compiler can't
//! do their "create a state machine" magic. In the end you can't have `.await` statements in non-async
//! functions.
//!
//! This library creates a special stack for executing the JIT code, so it's possible to suspend it at any
//! point of the execution. Once you pass it a closure inside [AsyncWormhole::new](struct.AsyncWormhole.html#method.new)
//! you will get back a future that you can `.await` on. The passed in closure is going to be executed on a
//! new stack.
//!
//! ## Example
//!
//! ```rust
//! use async_wormhole::{AsyncWormhole, AsyncYielder};
//! use switcheroo::stack::*;
//!
//! // non-async function
//! #[allow(improper_ctypes_definitions)]
//! extern "C" fn non_async(mut yielder: AsyncYielder<u32>) -> u32 {
//! 	// Suspend the runtime until async value is ready.
//! 	// Can contain .await calls.
//!     yielder.async_suspend(async { 42 })
//! }
//!
//! fn main() {
//!     let stack = EightMbStack::new().unwrap();
//!     let task = AsyncWormhole::<_, _, fn()>::new(stack, |yielder| {
//!         let result = non_async(yielder);
//!         assert_eq!(result, 42);
//!         64
//!     })
//!     .unwrap();
//!
//!     let outside = futures::executor::block_on(task);
//!     assert_eq!(outside, 64);
//! }
//! ```

use switcheroo::Generator;
use switcheroo::Yielder;

use std::cell::Cell;
use std::future::Future;
use std::io::Error;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};

pub use switcheroo::stack;

/// AsyncWormhole represents a Future that uses a generator with a separate stack to execute a closure.
///
/// It has the capability to .await on other Futures in the closure using the received
/// [AsyncYielder](struct.AsyncYielder). Once all Futures have been awaited on AsyncWormhole will resolve
/// to the return value of the provided closure.
///
/// For dealing with thread local storage
/// [AsyncWormhole::set_pre_post_poll](struct.AsyncWormhole.html#method.set_pre_post_poll) is provided.
///
/// Every time an executor polls AsyncWormhole, the `pre_post_poll` function will be called and every time
/// AsyncWormhole returns `Poll::Pending`, `pre_post_poll` will be called again. Between this two calls we
/// have a guarantee that the executor will not be able to move the execution to another thread, and we
/// can use this guarantee to our advantage in specific scenarios.
pub struct AsyncWormhole<'a, Stack, Output, P>
where
    Stack: stack::Stack + Send,
    P: FnMut() + Send,
{
    pub generator: Option<Cell<Generator<'a, Waker, Option<Output>, Stack>>>,
    pre_post_poll: Option<P>,
}

impl<'a, Stack, Output, P> AsyncWormhole<'a, Stack, Output, P>
where
    Stack: stack::Stack + Send,
    P: FnMut() + Send,
{
    /// Returns a new AsyncWormhole, using the passed `stack` to execute the closure `f` on.
    /// The closure will not be executed right away, only if you pass AsyncWormhole to an
    /// async executor (.await on it)
    pub fn new<F>(stack: Stack, f: F) -> Result<Self, Error>
    where
        F: FnOnce(AsyncYielder<Output>) -> Output + 'a + Send,
    {
        let generator = Generator::new(stack, |yielder, waker| {
            let async_yielder = AsyncYielder::new(yielder, waker);
            let finished = Some(f(async_yielder));
            yielder.suspend(finished);
        });

        Ok(Self {
            generator: Some(Cell::new(generator)),
            pre_post_poll: None,
        })
    }

    /// Every time the executor polls `AsyncWormhole` we may end up on another thread, here we can set a function
    /// that swaps some thread local storage and a context that can travel with `AsyncWormhole` between threads.
    pub fn set_pre_post_poll(&mut self, f: P) {
        self.pre_post_poll = Some(f);
    }

    /// Get the stack from the internal generator.
    pub fn stack(mut self) -> Stack {
        let generator = self.generator.take().unwrap().into_inner();
        // If the generator didn't finish yet, the stack is going to be unwinded on drop().
        // Fire a last pre_post_poll before this happens.
        if generator.started() && !generator.finished() {
            if let Some(pre_post_poll) = &mut self.pre_post_poll {
                pre_post_poll();
            }
        }
        generator.stack()
    }
}

impl<'a, Stack, Output, P> Drop for AsyncWormhole<'a, Stack, Output, P>
where
    Stack: stack::Stack + Send,
    P: FnMut() + Send,
{
    fn drop(&mut self) {
        // Dropping a generator can cause an unwind and execute code inside of the separate context.
        // In this regard it's similar to a `poll` call and we need to fire pre and post poll hooks.
        // Note, that we **don't** do a last `post_poll` call once the generator is dropped.
        if let Some(pre_post_poll) = &mut self.pre_post_poll {
            if let Some(generator) = self.generator.as_mut() {
                if generator.get_mut().started() && !generator.get_mut().finished() {
                    pre_post_poll()
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct AsyncYielder<'a, Output> {
    yielder: &'a Yielder<Waker, Option<Output>>,
    waker: Waker,
}

impl<'a, Output> AsyncYielder<'a, Output> {
    pub(crate) fn new(yielder: &'a Yielder<Waker, Option<Output>>, waker: Waker) -> Self {
        Self { yielder, waker }
    }
}
