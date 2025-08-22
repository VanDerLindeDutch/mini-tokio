use crate::io::epoll::{Event, MiniEpoll, LOCAL_EPOLL};
use crate::io::net::TcpStream;
use std::io;
use std::io::{ErrorKind, Write};
use std::os::fd::AsRawFd;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

pub struct TcpStreamWriter<'a, 'b> {
    writer: &'a mut TcpStream,
    epoll: Arc<MiniEpoll>,
    buf: &'b [u8],
}

impl TcpStreamWriter<'_, '_> {
    pub fn new<'a, 'b>(writer: &'a mut TcpStream, buf: &'b [u8]) -> TcpStreamWriter<'a, 'b> {
        TcpStreamWriter {
            writer,
            epoll: LOCAL_EPOLL.clone(),
            buf,
        }
    }
    fn split_borrow(&mut self) -> (&mut TcpStream, &[u8]) {
        (self.writer, self.buf)
    }
}

impl Future for TcpStreamWriter<'_, '_> {
    type Output = io::Result<usize>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let (strem, buf) = self.split_borrow();
        // println!("try to write");
        match strem.inner.write(buf) {
            Ok(v) => {
                Poll::Ready(Ok(v))
            }
            Err(er) if er.kind() == ErrorKind::WouldBlock => unsafe {
                // println!("wr  err {}", er);
                /*if self.writer.accepted {
                    return Poll::Pending;
                }*/
                // self.writer.accepted = true;
                self.epoll.clone().accept(self.writer.inner.as_raw_fd(), cx.waker().clone(), Event::Write).expect("TODO: panic message");
                Poll::Pending
            }
            Err(er) => {
                Poll::Ready(Err(er))
            }
        }
    }
}