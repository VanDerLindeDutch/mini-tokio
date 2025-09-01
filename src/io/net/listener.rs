use std::io::ErrorKind;
use crate::io::epoll::{Event, MiniEpoll, LOCAL_EPOLL};
use crate::io::net::TcpStream;
use std::os::fd::AsRawFd;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

pub struct TcpListener {
    pub inner: std::net::TcpListener,
    // accepted: bool,
}


pub struct TcpListenerAcceptor<'a> {
    epoll: Arc<MiniEpoll>,
    lister: &'a mut TcpListener,

}

impl Future for TcpListenerAcceptor<'_> {
    type Output = std::io::Result<TcpStream>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.lister.inner.accept() {
            Ok(v) => {
                v.0.set_nonblocking(true).expect("TODO: panic message");
                Poll::Ready(Ok(TcpStream {
                    inner: v.0,
                    // accepted: false,
                }))
            }
            Err(err) if err.kind() == ErrorKind::WouldBlock => unsafe {
                /*if self.lister.accepted {
                    return Poll::Pending;
                }*/
                // self.lister.accepted = true;
                self.epoll.clone().accept(self.lister.inner.as_raw_fd(), cx.waker().clone(), Event::Read).expect("TODO: panic message");
                Poll::Pending
            }
            Err(err) => {
                Poll::Ready(Err(err))
            }
        }
    }
}


impl TcpListener {
    pub fn new(innner: std::net::TcpListener) -> Self {
        TcpListener {
            inner: innner,
            // accepted: false,
        }
    }
    pub fn accept(&mut self) -> TcpListenerAcceptor<'_> {
        TcpListenerAcceptor {
            epoll: LOCAL_EPOLL.clone(),
            lister: self,

        }
    }
}
