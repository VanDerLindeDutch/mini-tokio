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
use libc::{epoll_ctl, epoll_wait, AF_INET, EPOLLIN, EPOLL_CTL_ADD, SOCK_STREAM};


pub struct MiniEpoll {
    map: Arc<Mutex<HashMap<u64, TcpListener>>>,
    key: Arc<AtomicU64>
}
// #[cfg(unix)]
impl MiniEpoll {
    unsafe fn some(&self) -> Result<(), Error> {
        let listener = TcpListener::bind("127.0.0.1:8000").unwrap();
        listener.set_nonblocking(true)?;

        let listener_fd = listener.as_raw_fd();
        let sock_listen_fd = libc::socket(AF_INET, SOCK_STREAM, 0);
        let mut events = [libc::epoll_event { events: 0, u64: 0 }; 100];
        let mut ev = libc::epoll_event { events: 0, u64: 0 };
        let epollfd = libc::epoll_create1(0);
        ev.events = EPOLLIN as u32;
        ev.u64 = self.key.fetch_add(1, SeqCst);
        self.map.lock().unwrap().insert(ev.u64, listener);
        epoll_ctl(epollfd, EPOLL_CTL_ADD, sock_listen_fd, &mut ev);
        loop {
            let new_events = epoll_wait(epollfd, &mut events[0], 100, -1);
            for ev in events {
                let r = self.map.lock().unwrap().get(ev.u64).unwrap();
                r.accept()
            }
        }
    }
}