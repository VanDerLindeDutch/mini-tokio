use std::io;
use std::io::{Error, Read};
use std::os::fd::AsRawFd;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use crate::net::epoll::MiniEpoll;

pub mod epoll;

pub struct TcpStream {
    pub inner: std::net::TcpStream,
    accepted: bool,
}

impl TcpStream {
    pub fn async_read<'a, 'b>(&'a mut self, buf: &'b mut [u8], epol: Arc<MiniEpoll>) -> TcpStreamReader<'a, 'b>{
        println!("try to read");
        TcpStreamReader{
            reader: self,
            epoll: epol,
            buf: buf,
        }
    }
}
pub struct TcpStreamReader<'a, 'b> {
    reader: &'a mut  TcpStream,
    epoll: Arc<MiniEpoll>,
    buf: &'b mut[u8]
}


impl TcpStreamReader<'_, '_> {
    fn split_borrow(&mut self)->(&mut TcpStream, &mut [u8]) {
        (self.reader, self.buf)
    }
}

impl Future for TcpStreamReader<'_, '_> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let (strem, buf) = self.split_borrow();
        println!("try to read");
        match strem.inner.read(buf) {
            Ok(v) => {
                println!("read {}", v);
                Poll::Ready(())
            }
            Err(er) => unsafe {
                println!("err {:?}", er);
                if self.reader.accepted {
                    return Poll::Pending;
                }
                self.reader.accepted = true;
                self.epoll.clone().accept(self.reader.inner.as_raw_fd(), cx.waker().clone()).expect("TODO: panic message");
                Poll::Pending
            }
        }
    }
}

pub struct TcpListener {
    inner: std::net::TcpListener,
    accepted: bool,
}


pub struct TcpListenerAcceptor<'a> {
    epoll: Arc<MiniEpoll>,
    lister: &'a mut  TcpListener,

}

impl Future for TcpListenerAcceptor<'_> {
    type Output = TcpStream;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.lister.inner.accept() {
            Ok(v) => {
                v.0.set_nonblocking(true).expect("TODO: panic message");
                Poll::Ready(TcpStream{
                    inner: v.0,
                    accepted: false,
                })
            }
            Err(er) => unsafe {
                if self.lister.accepted {
                   return Poll::Pending;
                }
                self.lister.accepted = true;
                self.epoll.clone().accept(self.lister.inner.as_raw_fd(), cx.waker().clone()).expect("TODO: panic message");
                Poll::Pending
            }
        }
    }
}


impl TcpListener {
    pub fn accept(&mut self, epol: Arc<MiniEpoll>)->TcpListenerAcceptor {
        TcpListenerAcceptor{
            epoll: epol,
            lister: self,

        }
    }
}

pub fn accept(adr: &str) -> io::Result<TcpListener> {
    let listener = std::net::TcpListener::bind(adr)?;
    listener.set_nonblocking(true)?;
    Ok(TcpListener {
        inner: listener,
        accepted: false,
    })
}

