use std::collections::HashMap;
use std::fmt::Debug;
use std::io::Error;
use std::iter::Map;
use std::net::{SocketAddr, SocketAddrV4, TcpListener, TcpStream};
use std::os::fd::{AsRawFd, RawFd};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering::SeqCst;
use std::task::Waker;
use std::thread;
use libc::{epoll_ctl, epoll_wait, AF_INET, EPOLLIN, EPOLL_CTL_ADD, SOCK_STREAM};


pub struct MiniEpoll {
    epollfd: libc::c_int,
    map: Mutex<HashMap<RawFd, Waker>>,
    key: AtomicU64,
    // events: [libc::epoll_event; 100],
}


#[cfg(unix)]
impl MiniEpoll {

    pub fn new()->Arc<Self> {
        let epollfd = unsafe { libc::epoll_create1(0) };
        let out = Arc::new(Self{
            epollfd,
            map: Default::default(),
            key: Default::default(),
        });
        let cloned_out = out.clone();
        thread::spawn(move || {
            unsafe {
                let mut events = [libc::epoll_event { events: 0, u64: 0 }; 100];
                loop {
                    let new_events = epoll_wait(epollfd, &mut events[0], 100, -1);
                    for ev in events {
                        if ev.u64 == 0 {
                            continue
                        }
                        let to_print = ev.u64;
                        let event = ev.events;

                        if !cloned_out.map.lock().unwrap().contains_key(&(ev.u64 as i32)) {
                            continue
                        }
                        println!("getted {} {}", to_print, event);
                        let r = cloned_out.map.lock().unwrap().remove(&(ev.u64 as i32)).unwrap();
                        r.wake();

                    }
                    println!("end");
                }

            }
        });
        out
    }

    pub unsafe  fn accept(self: Arc<Self>, fd: RawFd, waker: Waker) -> Result<(), Error> {
        // let sock_listen_fd = libc::socket(AF_INET, SOCK_STREAM, 0);
        self.map.lock().unwrap().insert(fd, waker);
        let mut ev = libc::epoll_event { events: EPOLLIN as u32, u64: fd as u64 };
        let to_print = ev.u64;
        println!("insert {}", to_print);
        epoll_ctl(self.epollfd, EPOLL_CTL_ADD, fd, &mut ev);
        Ok(())
    }
    unsafe fn some(self: Arc<Self>) -> Result<(), Error> {
        let listener = TcpListener::bind("127.0.0.1:8000").unwrap();
        listener.set_nonblocking(true)?;

        let listener_fd = listener.as_raw_fd();
        let sock_listen_fd = libc::socket(AF_INET, SOCK_STREAM, 0);
        //

       /*
        ev.events = EPOLLIN as u32;
        ev.u64 = self.key.fetch_add(1, SeqCst);
        self.map.lock().unwrap().insert(ev.u64, listener);
        epoll_ctl(epollfd, EPOLL_CTL_ADD, sock_listen_fd, &mut ev);*/

        Ok(())
    }
}