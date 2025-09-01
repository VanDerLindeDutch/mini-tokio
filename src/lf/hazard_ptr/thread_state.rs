use std::sync::atomic::AtomicPtr;
use crate::lf::hazard_ptr::MAX_HAZARD_PTRS;


pub struct ThreadState<T> {
    pub slots: [AtomicPtr<()>; MAX_HAZARD_PTRS],
    pub retired_ptrs: Vec<*const T>
}

impl<T> Default for ThreadState<T> {
    fn default() -> Self {
        Self{
            slots: Default::default(),
            retired_ptrs: vec![],
        }
    }
}


/*impl RetiredPtr {
    fn retired<T>(ptr: *mut T) ->Self {


    }
}*/