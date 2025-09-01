use std::pin::Pin;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::task::Poll::Ready;
use std::task::{Context, Poll, Waker};


pub struct SimpleMutex {
    ready: AtomicU64,
    buf: Arc<Mutex<Vec<Waker>>>,
}

pub struct Awaiter<'a> {
    mutex: &'a SimpleMutex
}



impl SimpleMutex {
    pub fn new() -> Self {
        SimpleMutex { ready: Default::default(), buf: Arc::new(Default::default()) }
    }

    pub fn  lock(&self) -> Awaiter<'_>{
        Awaiter{
            mutex: &self,
        }
    }
    pub fn unlock(&self) {
        let mut l = self.buf.lock().unwrap();
        self.ready.store(0, SeqCst);
        if let Some(w) = l.pop() {
            std::mem::drop(l);
            w.wake();
        }

    }
}

impl Future for Awaiter<'_> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.mutex.ready.swap(1, Ordering::SeqCst) == 0 {
            return Ready(());
        }
        let mut l = self.mutex.buf.lock().unwrap();
        if self.mutex.ready.swap(1, Ordering::SeqCst) == 0 {
            return Ready(());
        }
        l.push(cx.waker().clone());
        Poll::Pending
    }
}