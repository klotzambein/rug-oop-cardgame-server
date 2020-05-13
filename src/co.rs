use std::task::{RawWaker, RawWakerVTable, Waker};
use std::{future::Future, task::{Context, Poll}};

use lazy_static::lazy_static;

struct TestFuture(u8);

impl Future for TestFuture {
    type Output = ();
    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> Poll<<Self as Future>::Output> {
        if self.0 == 0 {
            return Poll::Ready(());
        } else {
            self.0 -= 1;
            return Poll::Pending;
        }
    }
}

pub async fn test() {
    TestFuture(0).await;
    TestFuture(0).await;
}

pub fn execute<T>(fut: impl Future<Output = T>) -> T {
    let mut pinned = Box::pin(fut);
    let mut ctx = Context::from_waker(&NULL_WAKER);
    loop {
        let result = pinned.as_mut().poll(&mut ctx);
        if let Poll::Ready(result) = result {
            return  result;
        }
    }
}

lazy_static! {
    // Safety: The waker points to a vtable with functions that do nothing. Doing
    // nothing is memory-safe.
    pub static ref NULL_WAKER: Waker = unsafe { Waker::from_raw(RAW_WAKER) };
}
const RAW_WAKER: RawWaker = RawWaker::new(std::ptr::null(), &VTABLE);
const VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

fn clone(_: *const ()) -> RawWaker {
    RAW_WAKER
}
fn wake(_: *const ()) {}
fn wake_by_ref(_: *const ()) {}
fn drop(_: *const ()) {}
