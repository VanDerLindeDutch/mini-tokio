#![feature(mpmc_channel)]
#![feature(lazy_cell_into_inner)]

use std::cell::UnsafeCell;
use std::sync::{Arc, LazyLock};
use std::thread::sleep;
use std::time::Duration;
use crate::executor::{EXECUTOR, S};
use crate::net::epoll::MiniEpoll;
use crate::net::TcpListener;

mod executor;
mod queue;
mod mini_mutex;
mod net;

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
            unsafe { *(ptr as *mut i32) += 1 };
            println!("{}", unsafe { *(ptr as *const i32) });
            m.unlock();
            // println!("unlock");
        });
    });

    EXECUTOR.add(async {
        let EPOLL = MiniEpoll::new();
        let mut listener = net::accept("127.0.0.1:8080").unwrap();
        loop {
            let mut acceptor = listener.accept(EPOLL.clone()).await;
            let cloned_epoll = EPOLL.clone();
            EXECUTOR.add(async move {
                println!("{:?}", acceptor.inner.peer_addr().unwrap());
                let mut buf = [0u8;1024];
                acceptor.async_read(&mut buf, cloned_epoll).await;
                println!("{:?}", String::from_utf8_lossy(&buf));
            });

        }
    });
    EXECUTOR.add(async {
        let EPOLL = MiniEpoll::new();
        let mut listener = net::accept("127.0.0.1:8081").unwrap();
        loop {
            let acceptor = listener.accept(EPOLL.clone()).await;
            println!("{:?}", acceptor.inner.peer_addr().unwrap());
        }
    });

    EXECUTOR.add(async {
        println!("YEAH")
    });
    sleep(Duration::from_secs(10000));
    println!("{}", unsafe { *(i.get()) });
    // EXECUTOR.wait();

}
