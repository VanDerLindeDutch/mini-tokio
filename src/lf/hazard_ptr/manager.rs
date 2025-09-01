use crate::lf::hazard_ptr::mutator::Mutator;
use crate::lf::hazard_ptr::stack::LockFreeStack;
use crate::lf::hazard_ptr::thread_state::ThreadState;
use crate::syscall;
use std::ffi::c_void;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicUsize};

// thread_local! {static MANAGER_STATE: RefCell<HashMap<u64, *mut ThreadState>> =  {RefCell::new(HashMap::new())}}
pub struct Manager<T> {
    key: libc::pthread_key_t,
    pub thread_state_stack: LockFreeStack<*mut ThreadState<T>>,
    pub is_retiring: AtomicBool,
    pub size: AtomicUsize,
}

impl<T> Manager<T> {
    pub fn new() -> Pin<Box<Self>> {
        // *q = Manager{ key: 0 };
        let mut out = Box::pin(Manager {
            key: 0,
            thread_state_stack: LockFreeStack::new(),
            is_retiring: AtomicBool::from(false),
            size: AtomicUsize::new(0),
        });
        syscall!(pthread_key_create(&mut out.key, None)).expect("pthread_key_create");
        out
    }

    pub fn make_mutator(&self) -> Mutator<T> {

        // syscall!();

        let thread = match unsafe { libc::pthread_getspecific(self.key) } {
            ptr if ptr.is_null() => {
                let thread = Box::into_raw(Box::new(ThreadState::default()));
                self.thread_state_stack.push(thread);
                syscall!(pthread_setspecific(self.key, thread as *const c_void)).expect("pthread_setspecific");
                thread
            }
            ptr => ptr as *mut ThreadState<T>
        };

        Mutator::new(self, thread)
    }
}

impl<T> Drop for Manager<T> {
    fn drop(&mut self) {
        while let Some(v) = self.thread_state_stack.try_pop() {
            unsafe {
                let _ = (*(*v).value).retired_ptrs.iter().map(|x| { std::ptr::drop_in_place(*x as *mut T) });
            }
        }
        syscall!(pthread_key_delete(self.key)).expect("pthread_key_delete");
    }
}