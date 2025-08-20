use std::collections::VecDeque;
use std::sync::{Condvar, Mutex};
// static EXECUTOR: BlockingQueue = BlockingQueue{ buf: (Default::default(), Default::default()) };
struct BlockingQueue<T> {
    buf: (Mutex<VecDeque<Option<T>>>, Condvar)
}





