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
use std::{panic, thread};
use libc::{c_int, epoll_ctl, epoll_wait, AF_INET, EPOLLIN, EPOLLONESHOT, EPOLLOUT, EPOLL_CTL_ADD, EPOLL_CTL_MOD, SOCK_STREAM};


pub struct MiniEpoll {
    epollfd: libc::c_int,
    map: Mutex<HashMap<RawFd, Waker>>,
    // events: [libc::epoll_event; 100],
}


#[cfg(unix)]
impl MiniEpoll {
    pub fn new() -> Arc<Self> {
        let epollfd = unsafe { libc::epoll_create1(0) };
        let out = Arc::new(Self {
            epollfd,
            map: Default::default(),
        });
        let cloned_out = out.clone();
        thread::spawn(move || {
            unsafe {
                let mut events = [libc::epoll_event { events: 0, u64: 0 }; 100];
                loop {
                    let new_events = epoll_wait(epollfd, &mut events[0], 100, -1);

                    for i in 0..new_events as usize {
                        let ev = &events[i];
                        let to_print = ev.u64;
                        let event = ev.events;

                        // tracing::info!("first getted {} {}", to_print, event);

                        // tracing::info!("secobd getted {} {}", to_print, event);
                        // println!("getted {} {}", to_print, event);
                        if !cloned_out.map.lock().unwrap().contains_key(&(ev.u64 as i32)) {
                            continue;
                        }
                        tracing::info!("finally getted {} {}", to_print, event);
                        let r = cloned_out.map.lock().unwrap().get(&(ev.u64 as i32)).unwrap().clone();
                        r.wake();
                    }
                    tracing::info!("end");
                }
            }
        });
        out
    }


    pub unsafe fn accept(self: Arc<Self>, fd: RawFd, waker: Waker, event: Event) -> Result<(), Error> {
        // let sock_listen_fd = libc::socket(AF_INET, SOCK_STREAM, 0);
        let flag = match self.map.lock().unwrap().insert(fd, waker) {
            None => EPOLL_CTL_ADD,
            Some(_) => EPOLL_CTL_MOD
        };
        let mut ev = libc::epoll_event { events: (event.to_epoll()) as u32, u64: fd as u64 };
        let to_print = ev.u64;
        tracing::info!("insert {}", to_print);
        let res = epoll_ctl(self.epollfd, flag, fd, &mut ev);
        if res == -1 {
            panic!("{}", std::io::Error::last_os_error())
        }

        Ok(())
    }
}

pub enum Event {
    Read,
    Write
}

impl Event {
    fn to_epoll(&self) -> c_int {
        match self {
            Event::Read => EPOLLIN|EPOLLONESHOT,
            Event::Write => EPOLLOUT|EPOLLONESHOT
        }
    }
}