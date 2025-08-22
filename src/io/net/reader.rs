use crate::io::epoll::{Event, MiniEpoll, LOCAL_EPOLL};
use crate::io::net::TcpStream;
use std::io;
use std::io::{ErrorKind, Read};
use std::os::fd::AsRawFd;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

pub struct TcpStreamReader<'a, 'b> {
    reader: &'a mut TcpStream,
    epoll: Arc<MiniEpoll>,
    buf: &'b mut [u8],
}


impl TcpStreamReader<'_, '_> {
    pub fn new<'a, 'b>(reader: &'a mut TcpStream, buf: &'b mut [u8]) -> TcpStreamReader<'a, 'b> {
        TcpStreamReader{
            reader,
            epoll: LOCAL_EPOLL.clone(),
            buf,
        }
    }
    fn split_borrow(&mut self) -> (&mut TcpStream, &mut [u8]) {
        (self.reader, self.buf)
    }
}

impl Future for TcpStreamReader<'_, '_> {
    type Output = io::Result<usize>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let (strem, buf) = self.split_borrow();
        // tracing::info!("try to read");
        match strem.inner.read(buf) {
            Ok(v) => {
                // tracing::info!("read {}", v);
                Poll::Ready(Ok(v))
            }
            Err(er) if er.kind() == ErrorKind::WouldBlock => unsafe {
                /*tracing::info!("err {:?}", er);
               /* if self.reader.accepted {
                    return Poll::Pending;
                }*/*/
                // self.reader.accepted = true;
                self.epoll.clone().accept(self.reader.inner.as_raw_fd(), cx.waker().clone(), Event::Read).expect("TODO: panic message");
                Poll::Pending
            }
            Err(er) => {
                Poll::Ready(Err(er))
            }
        }
    }
}