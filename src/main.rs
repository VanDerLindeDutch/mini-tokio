#![feature(mpmc_channel)]
#![feature(lazy_cell_into_inner)]

use std::cell::UnsafeCell;
use std::io::Write;
use std::sync::{Arc, LazyLock};
use std::thread::sleep;
use std::time::Duration;
use tracing_subscriber::fmt::format::FmtSpan;
use crate::executor::{EXECUTOR, S};
// use crate::net::epoll::MiniEpoll;
use crate::net::{MiniEpoll, TcpListener};

mod executor;
mod queue;
mod mini_mutex;
mod net;

fn main() {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .with_max_level(tracing_subscriber::filter::LevelFilter::DEBUG)
        .init();

    let i = UnsafeCell::new(0);
    let M = Arc::new(mini_mutex::SimpleMutex::new());
    let ptr = i.get() as usize;
    let v = vec![0u8; 100];
    let q = v.iter();

    /*(0..1000).into_iter().for_each(|x| {
        let m = M.clone();
        EXECUTOR.add(async move {
            // println!("try to lock");
            m.lock().await;
            // println!("got lock");
            unsafe { *(ptr as *mut i32) += 1 };
            // println!("{}", unsafe { *(ptr as *const i32) });
            m.unlock();
            // println!("unlock");
        });
    });*/
    let EPOLL = MiniEpoll::new();
    let cloned_epoll = EPOLL.clone();
    EXECUTOR.add(async move {

        let mut listener = net::accept("127.0.0.1:8080").unwrap();
        loop {
            let mut acceptor = listener.accept(cloned_epoll.clone()).await;
            let cloned_epoll = cloned_epoll.clone();
            EXECUTOR.add(async move {
                tracing::info!("{:?}", acceptor.inner.peer_addr().unwrap());
                loop {
                    let mut buf = [0u8; 1024];
                    acceptor.async_read(&mut buf, cloned_epoll.clone()).await;

                    tracing::info!("{:?}", String::from_utf8_lossy(&buf));
                }
            });
        }
    });
    let cloned_epoll = EPOLL.clone();
    EXECUTOR.add(async move {
        std::thread::sleep(Duration::from_secs(1));
        // let EPOLL = MiniEpoll::new();
        let mut writer = net::connect("127.0.0.1:8080").unwrap();
        loop {
            let cloned_epoll = cloned_epoll.clone();
            let to_write = "some test async bytes";
            writer.async_write(to_write.as_bytes(), cloned_epoll.clone()).await;
            tracing::info!("writed!");
            std::thread::sleep(Duration::from_secs(5));
            // println!("{:?}", acceptor.inner.peer_addr().unwrap());
        }
    });

    EXECUTOR.add(async {
        println!("YEAH")
    });
    sleep(Duration::from_secs(10000));
    println!("{}", unsafe { *(i.get()) });
    // EXECUTOR.wait();

}
