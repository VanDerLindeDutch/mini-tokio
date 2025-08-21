use std::io;
use std::io::{Error, ErrorKind, Read, Write};
use std::net::ToSocketAddrs;
use std::os::fd::AsRawFd;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
// use crate::net::epoll::MiniEpoll;

mod epoll;

pub use epoll::MiniEpoll;
use crate::net::epoll::Event;

pub struct TcpStream {
    pub inner: std::net::TcpStream,
    accepted: bool,
}

impl TcpStream {
    pub fn async_read<'a, 'b>(&'a mut self, buf: &'b mut [u8], epol: Arc<MiniEpoll>) -> TcpStreamReader<'a, 'b>{
        // println!("try to read");
        TcpStreamReader{
            reader: self,
            epoll: epol,
            buf: buf,
        }
    }

    pub fn async_write<'a, 'b>(&'a mut self, buf: &'b [u8], epol: Arc<MiniEpoll>) -> TcpStreamWriter<'a, 'b>{
        // println!("try to read");
        TcpStreamWriter{
            writer: self,
            epoll: epol,
            buf: buf,
        }
    }
}


pub struct TcpStreamWriter<'a, 'b> {
    writer: &'a mut  TcpStream,
    epoll: Arc<MiniEpoll>,
    buf: &'b [u8]
}




impl crate::net::TcpStreamWriter<'_, '_> {
    fn split_borrow(&mut self)->(&mut TcpStream, &[u8]) {
        (self.writer, self.buf)
    }
}

impl Future for crate::net::TcpStreamWriter<'_, '_> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let (strem, buf) = self.split_borrow();
        // println!("try to read");
        match strem.inner.write(buf) {
            Ok(v) => {
                // println!("read {}", v);
                Poll::Ready(())
            }
            Err(er) if er.kind() == ErrorKind::WouldBlock => unsafe {

                if self.writer.accepted {
                    return Poll::Pending;
                }
                self.writer.accepted = true;
                self.epoll.clone().accept(self.writer.inner.as_raw_fd(), cx.waker().clone(), Event::Write).expect("TODO: panic message");
                Poll::Pending
            }
            Err(er) => {
                panic!("{}", er)
            }
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
        tracing::info!("try to read");
        match strem.inner.read(buf) {
            Ok(v) => {
                tracing::info!("read {}", v);
                Poll::Ready(())
            }
            Err(er) => unsafe {
                tracing::info!("err {:?}", er);
               /* if self.reader.accepted {
                    return Poll::Pending;
                }*/
                self.reader.accepted = true;
                self.epoll.clone().accept(self.reader.inner.as_raw_fd(), cx.waker().clone(), Event::Read).expect("TODO: panic message");
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
                self.epoll.clone().accept(self.lister.inner.as_raw_fd(), cx.waker().clone(), Event::Read).expect("TODO: panic message");
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

pub fn accept<T:ToSocketAddrs>(adr: T) -> io::Result<TcpListener> {
    let listener = std::net::TcpListener::bind(adr)?;
    listener.set_nonblocking(true)?;
    Ok(TcpListener {
        inner: listener,
        accepted: false,
    })
}

pub fn connect<T:ToSocketAddrs>(adr: T)-> io::Result<TcpStream> {
    let out = std::net::TcpStream::connect(adr)?;
    out.set_nonblocking(true)?;
    Ok(TcpStream{ inner: out, accepted: false })
}

