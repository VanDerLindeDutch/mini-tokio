#![feature(mpmc_channel)]
#![feature(lazy_cell_into_inner)]

use crate::executor::EXECUTOR;
use std::cell::UnsafeCell;
use std::sync::{Arc};
use std::thread::sleep;
use std::time::Duration;
use tracing_subscriber::fmt::format::FmtSpan;
// use crate::net::epoll::MiniEpoll;

mod executor;
mod queue;
mod mini_mutex;
mod io;
mod helper;
mod lf;
fn main() {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .with_max_level(tracing_subscriber::filter::LevelFilter::DEBUG)
        .init();

    let i = UnsafeCell::new(0);
    let mutex = Arc::new(mini_mutex::SimpleMutex::new());
    let ptr = i.get() as usize;

    (0..1000).into_iter().for_each(|_| {
        let m = mutex.clone();
        EXECUTOR.add(async move {
            // println!("try to lock");
            m.lock().await;
            // io::sleep(Duration::from_secs(10)).await;
            // println!("got lock");
            unsafe { *(ptr as *mut i32) += 1 };
            // println!("jopa  {}", unsafe { *(ptr as *const i32) });
            m.unlock();
            // println!("unlock");
        });
    });

    // let cloned_epoll = EPOLL.clone();
    EXECUTOR.add(async move {

        let mut listener = io::bind("127.0.0.1:8080").unwrap();
        loop {
            let mut acceptor = listener.accept().await.expect("listener.accept");
            // let cloned_epoll = cloned_epoll.clone();
            EXECUTOR.add(async move {
                // tracing::info!("{:?}", acceptor.inner.peer_addr().unwrap());
                loop {
                    let mut buf = [0u8; 1024];
                    let read = acceptor.async_read(&mut buf).await;
                    match read {
                        Ok(v) if v == 0 => {break;}
                        Err(err) => {tracing::error!("{:?}", err)}
                        _ => {}
                    }


                    tracing::info!("{:?}", String::from_utf8_lossy(&buf));
                }
            });
        }
    });
    // let cloned_epoll = EPOLL.clone();
    EXECUTOR.add(async move {
        std::thread::sleep(Duration::from_secs(1));
        // let EPOLL = MiniEpoll::new();

        let mut writer = io::connect("127.0.0.1:8080").unwrap();

        for _ in 0..20 {
            // writer.inner
            // let cloned_epoll = cloned_epoll.clone();
            let to_write = "some test async bytes".repeat(1000);
            writer.async_write(to_write.as_bytes()).await.expect("writer.async_write()");
            tracing::info!("writed!");
            io::timer::sleep(Duration::from_secs(1)).await;
            // println!("{:?}", acceptor.inner.peer_addr().unwrap());
        }
        io::timer::sleep(Duration::from_secs(20)).await;
        for _ in 0..20 {
            // writer.inner
            // let cloned_epoll = cloned_epoll.clone();
            let to_write = "some test async bytes".repeat(1000);
            writer.async_write(to_write.as_bytes()).await.expect("writer.async_write()");
            tracing::info!("writed!");
            io::timer::sleep(Duration::from_secs(1)).await;
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
