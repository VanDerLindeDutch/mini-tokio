use std::ptr::null_mut;
use std::sync::atomic::AtomicPtr;
use std::sync::atomic::Ordering::SeqCst;

pub struct PtrGuard {
    pub ptr: *const AtomicPtr<()>
}

impl PtrGuard {
    pub fn protect<T>(&self, in_ptr: &AtomicPtr<T>) -> *mut T {
        let mut v = in_ptr.load(SeqCst);
        loop {
            unsafe { (*self.ptr).store(v as *mut (), SeqCst); }

            if v == in_ptr.load(SeqCst) {
                break;
            }
            v =  in_ptr.load(SeqCst);
        }
        v
    }

    pub fn reset(&self) {
        unsafe { (&*self.ptr).store(null_mut(), SeqCst); }
    }
}