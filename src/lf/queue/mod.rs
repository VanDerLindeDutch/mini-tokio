mod test;

use crate::lf::hazard_ptr::manager::Manager;
use std::ops::Deref;
use std::pin::Pin;
use std::ptr::null_mut;
use std::sync::atomic::AtomicPtr;
use std::sync::atomic::Ordering::SeqCst;
// use crate::lf::manager;

pub struct LockFreeQueue<T> {
    head: AtomicPtr<Node<T>>,
    tail: AtomicPtr<Node<T>>,
    manager: Pin<Box<Manager<Node<T>>>>,
}



struct Node<T> {
    val: Option<T>,
    next: AtomicPtr<Node<T>>,
}

impl<T> Default for Node<T> {
    fn default() -> Self {
        Self{
            val: None,
            next: Default::default(),
        }
    }
}

impl<T> LockFreeQueue<T> {
    pub fn new() -> LockFreeQueue<T> {
        let dummy = Box::into_raw(Box::new(Node::default()));
        let out = LockFreeQueue {
            head: Default::default(),
            tail: Default::default(),
            manager: Manager::new(),
        };
        out.tail.store(dummy, SeqCst);
        out.head.store(dummy, SeqCst);
        out


    }
    pub fn push(&self, value: T) {
        let new_node = Box::into_raw(Box::new(Node { val: Some(value), next: AtomicPtr::new(null_mut()) }));
        let mut curr: *mut Node<T>;
        let mutator = self.manager.make_mutator();
        let top_guard = mutator.get_hazard_ptr(0);
        unsafe {
            loop {
                curr = top_guard.protect(&self.tail);

                if !(*curr).next.load(SeqCst).is_null() {
                    let _ = self.tail.compare_exchange_weak(curr, (*curr).next.load(SeqCst), SeqCst, SeqCst);
                    continue;
                }
                if let Ok(_) = (*curr).next.compare_exchange_weak(null_mut(), new_node, SeqCst, SeqCst) {
                    break;
                }
            }

            let _ = self.tail.compare_exchange(curr, (*curr).next.load(SeqCst), SeqCst, SeqCst);
        }
        top_guard.reset();
    }

    pub fn try_pop(&self) -> Option<T> {
        let mutator = self.manager.make_mutator();
        let (top_guard, back_guard) = (mutator.get_hazard_ptr(0), mutator.get_hazard_ptr(1));
        unsafe {
            loop {
                let h = back_guard.protect(&self.head);
                let next_h = top_guard.protect(&(*self.head.load(SeqCst)).next);
                if next_h.is_null() {
                    back_guard.reset();
                    top_guard.reset();
                    return None;
                }
                if let Ok(_) = self.head.compare_exchange_weak(h, (*h).next.load(SeqCst), SeqCst, SeqCst) {
                    let v = std::ptr::read(next_h);
                    back_guard.reset();
                    top_guard.reset();
                    mutator.retire(h);
                    return v.val;
                }
            }
        }
    }
}
pub struct LockFreeQueuePtr<T>(*const LockFreeQueue<T>);

impl<T> Deref for LockFreeQueuePtr<T> {
    type Target = LockFreeQueue<T>;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

unsafe impl<T> Send for LockFreeQueuePtr<T> {}