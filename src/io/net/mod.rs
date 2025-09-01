mod writer;
mod reader;
mod listener;

use crate::io::net::listener::TcpListener;
use crate::io::net::reader::TcpStreamReader;
use crate::io::net::writer::TcpStreamWriter;
use std::io;
use std::net::ToSocketAddrs;


pub struct TcpStream {
    inner: std::net::TcpStream,
    // accepted: bool,
}

impl TcpStream {
    pub fn async_read<'a, 'b>(&'a mut self, buf: &'b mut [u8]) -> TcpStreamReader<'a, 'b>{
        TcpStreamReader::new(self, buf)
    }

    pub fn async_write<'a, 'b>(&'a mut self, buf: &'b [u8]) -> TcpStreamWriter<'a, 'b>{
        TcpStreamWriter::new(self, buf)
    }
}




pub fn bind<T:ToSocketAddrs>(adr: T) -> io::Result<TcpListener> {
    let listener = std::net::TcpListener::bind(adr)?;
    listener.set_nonblocking(true)?;
    Ok(TcpListener::new(listener))
}

pub fn connect<T:ToSocketAddrs>(adr: T)-> io::Result<TcpStream> {
    let out = std::net::TcpStream::connect(adr)?;
    out.set_nonblocking(true)?;
    Ok(TcpStream{ inner: out })
}

