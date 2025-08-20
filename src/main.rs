#![feature(mpmc_channel)]
#![feature(lazy_cell_into_inner)]

use std::cell::UnsafeCell;
use std::sync::{Arc, LazyLock};
use std::thread::sleep;
use std::time::Duration;
use crate::executor::{EXECUTOR, S};

mod executor;
mod queue;
mod mini_mutex;
mod epoll;

fn main() {

    let i = UnsafeCell::new(0);
    let M = Arc::new(mini_mutex::SimpleMutex::new());
    let ptr = i.get() as usize;

    (0..1000).into_iter().for_each(|x| {
        let m = M.clone();
        EXECUTOR.add(async move {
            // println!("try to lock");
            m.lock().await;
            // println!("got lock");
            unsafe {*(ptr as *mut i32)+=1};
            println!("{}", unsafe{*(ptr as *const i32)});
            m.unlock();
            // println!("unlock");
        });
    });

    EXECUTOR.add(async {

    });

    EXECUTOR.add(async {
        println!("YEAH")
    });
    sleep(Duration::from_secs(1));
    println!("{}", unsafe{*(i.get())});

}
