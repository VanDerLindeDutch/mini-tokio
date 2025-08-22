use std::io::{Read, Write};
use std::net::ToSocketAddrs;
use std::os::fd::AsRawFd;
// use crate::net::epoll::MiniEpoll;

mod epoll;
mod timer;
mod net;

pub use net::{bind, connect};
pub use timer::sleep;

