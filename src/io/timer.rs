use crate::io::epoll::{Event, MiniEpoll, LOCAL_EPOLL};
use libc::CLOCK_MONOTONIC;
use std::os::raw::c_int;
use std::pin::Pin;
use std::ptr::null_mut;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;


pub fn sleep(duration: Duration) -> TimerAwaiter {
    let timer_fd = unsafe { libc::timerfd_create(CLOCK_MONOTONIC, 0) };
    let mut specs = libc::itimerspec {
        it_interval: libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        it_value: libc::timespec {
            tv_sec: (duration.as_millis() as i64 / 1000),
            tv_nsec: 0,
        },
    };
    unsafe { libc::timerfd_settime(timer_fd, 0, &mut specs, null_mut()); }
    TimerAwaiter { timer_fd, epoll: LOCAL_EPOLL.clone(), first_time: true }
}

pub struct TimerAwaiter {
    timer_fd: c_int,
    epoll: Arc<MiniEpoll>,
    first_time: bool,
}

impl Future for TimerAwaiter {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.first_time {
           return Poll::Ready(())
        }
        self.first_time = false;
        unsafe { self.epoll.clone().accept(self.timer_fd, cx.waker().clone(), Event::Read).expect("TODO: panic message"); }
        Poll::Pending
    }
}