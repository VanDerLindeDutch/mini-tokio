use crate::lf::hazard_ptr::guard::PtrGuard;
use crate::lf::hazard_ptr::manager::Manager;
use crate::lf::hazard_ptr::thread_state::ThreadState;
use crate::lf::hazard_ptr::MAX_HAZARD_PTRS;
use std::sync::atomic::Ordering::SeqCst;

// thread_local! {static STATE: Cell<*mut ThreadState> =  {Cell::new(null::<ThreadState>() as *mut ThreadState)}}

pub(crate) struct Mutator<T> {
    thread: *mut ThreadState<T>,
    manager_ptr: *const Manager<T>,
}
// static STACK: LockFreeStack<*mut ThreadState> = LockFreeStack::new();

impl<T> Mutator<T> {
    pub fn get_hazard_ptr(&self, index: usize) -> PtrGuard {
        unsafe { PtrGuard { ptr: &(*self.thread).slots[index] } }
    }

    pub fn retire(&self, obj: *mut T) {
        unsafe {
            (*self.thread).retired_ptrs.push(obj);
            if (*self.thread).retired_ptrs.len() < (*self.manager_ptr).size.load(SeqCst) * MAX_HAZARD_PTRS || (*self.manager_ptr).is_retiring.swap(true, SeqCst) {
                return;
            }
            let mut head = (*self.manager_ptr).thread_state_stack.try_pop();
            let mut plist = vec![];
            while let Some(v) = head {
                if v.is_null() {
                    break;
                }
                (*(*v).value).slots.iter().for_each(|x| {
                    match x.load(SeqCst) {
                        x if !x.is_null() => plist.push(x),
                        _ => {}
                    }
                });
                head = Some((*v).next);
            }
            let mut tmp_list = vec![];
            plist.sort();
            (*self.thread).retired_ptrs.iter().for_each(|x| {
                if let Ok(_) = tmp_list.binary_search(x) {
                    tmp_list.push(*x);
                } else {
                    std::ptr::drop_in_place(*x as *mut T );
                }
            });
            (*self.thread).retired_ptrs = tmp_list;
            (*self.manager_ptr).is_retiring.store(false, SeqCst)
        }
    }
    /* fn pop() -> Option<*const crate::lf::stack::Node<*mut ThreadState>> {
         .try_pop()
     }

     fn push(t: *mut ThreadState) {
         STACK.push(t);
     }*/
    pub fn new(manager_ptr: *const Manager<T>, thread: *mut ThreadState<T>) -> Self {
        Self {
            thread,
            manager_ptr,
        }
    }
}



