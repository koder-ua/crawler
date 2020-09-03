use std::sync::{Mutex, Arc, mpsc};
use std::sync::mpsc::{Receiver, Sender};


pub struct Queue<T> {
    send_ch: Sender<T>,
    recv_ch: Arc<Mutex<Receiver<T>>>,
}

impl<T> Queue<T> {
    pub fn new() -> Self {
        let (s, r) = mpsc::channel();
        Queue{
            send_ch: s,
            recv_ch: Arc::new(Mutex::new(r)) ,
        }
    }

    pub fn recv(&self) -> T {
        let v = self.recv_ch.lock().unwrap();
        return v.recv().unwrap();
    }

    pub fn send(&self, val: T) {
        return self.send_ch.send(val).unwrap();
    }

    pub fn clone(&self) -> Self {
        Queue{ send_ch: self.send_ch.clone(), recv_ch: self.recv_ch.clone() }
    }
}