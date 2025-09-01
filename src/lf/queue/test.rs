use crate::lf::queue::{LockFreeQueue, LockFreeQueuePtr};
use std::collections::HashSet;
use std::thread;

#[test]
pub fn simple() {
    let q = LockFreeQueue::new();
    let mut threads = vec![];
    for x in 0..100 {
        let q_ref = LockFreeQueuePtr(&q);
        threads.push(thread::spawn( move || {q_ref.push(x)}));
    }
    threads.into_iter().for_each(|x| {x.join().expect("")});
    let mut hash_set = HashSet::new();
    while let Some(v) = q.try_pop() {
        assert_eq!(hash_set.contains(&v), false);
        hash_set.insert(v);
    }

    assert_eq!(hash_set.len(),  100);

    assert_eq!(q.try_pop(), None);
}