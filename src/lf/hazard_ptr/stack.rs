use std::ptr::null_mut;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::atomic::Ordering::SeqCst;

pub struct Node<T> {
    pub value: T,
    pub next: *mut Node<T>,
}
pub struct LockFreeStack<T> {
    head: AtomicPtr<Node<T>>,
}

impl<T> LockFreeStack<T> {
    pub const fn new()-> LockFreeStack<T> {
        Self{
            head: AtomicPtr::new(null_mut()),
        }
    }
    pub fn push(&self, val: T) {
        // libc::pthread_setspecific()
        // auto new_node = new Node{std::move (val), head_.load()};
        let new_node = Box::into_raw(Box::new(Node { value: val, next: self.head.load(SeqCst) }));
        loop {
            unsafe {
                match self.head.compare_exchange_weak((*new_node).next, new_node, Ordering::SeqCst, SeqCst) {
                    // size.fetch_add(1);

                    Ok(_) => { return; }
                    Err(ptr) => { (*new_node).next = ptr }
                }
            }
        }
    }

    pub fn try_pop(&self) -> Option<*const Node<T>> {
        let mut h = self.head.load(SeqCst);
        loop {
            if h.is_null() {
                return None;
            }
            unsafe {
                match self.head.compare_exchange_weak(h, (*h).next, SeqCst, SeqCst) {
                    Ok(_) => return Some(h),
                    Err(ptr) => h = ptr
                }
            }
        }
    }
}
