use std::cell::RefCell;
use std::collections::VecDeque;
use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use std::pin::Pin;
use std::sync::{Arc, LazyLock, Mutex};
use std::task::{Context, Wake};
use std::thread::JoinHandle;
use std::time::Duration;


pub static EXECUTOR: LazyLock<ThreadPool> = LazyLock::new(|| {
    let mut out = ThreadPool { buf: std::sync::mpmc::channel(), threads: Default::default() };
    out.run();
    out
});


pub static S: LazyLock<String> = LazyLock::new(|| { "".to_string() });

pub type Fut = dyn Future<Output=()> + Send + Sync;

pub struct Task {
    sender: std::sync::mpmc::Sender<Arc<Task>>,
    fun: Mutex<Pin<Box<Fut>>>,
}
impl Debug for Task {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "jjj")
    }
}

impl Wake for Task {
    fn wake(self: Arc<Self>) {
        self.clone().sender.send(self).expect("TODO: panic message");
    }
}

pub struct ThreadPool {
    threads: RefCell<Vec<JoinHandle<()>>>,
    buf: (std::sync::mpmc::Sender<Arc<Task>>, std::sync::mpmc::Receiver<Arc<Task>>),
}

unsafe impl Send for ThreadPool {}
unsafe impl Sync for ThreadPool {}

pub fn spawn() {}
impl ThreadPool {
    pub fn run(&mut self) {
        /*if self.threads {
            unreachable!()
        }*/
        for _ in 0..1 {
            let recv = self.buf.1.clone();
            let h = std::thread::spawn(move || {

                // waker_ref()
                while let Ok(mut v) = recv.recv() {
                    // println!("new iter");
                    let w = &v.clone().into();
                    let mut ctx = Context::from_waker(w);
                    v.fun.lock().unwrap().as_mut().poll(&mut ctx);
                }
            });
            self.threads.get_mut().push(h);
        }
    }

    pub fn add<T: Future<Output=()> + Send + Sync + 'static>(&self, fun: T) {
        self.buf.0.send(Arc::new(Task { fun: Mutex::new(Box::pin(fun)), sender: self.buf.0.clone() })).unwrap();
    }


    pub fn wait(&mut self) {
        self.threads.take().into_iter().map(|x| { x.join() }).for_each(|x1| { x1.unwrap() })
    }
}