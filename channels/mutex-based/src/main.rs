use std::collections::VecDeque;
use std::sync::{Condvar, Mutex};
use std::thread::{self, sleep};
use std::time::Duration;

#[derive(Debug)]
struct Channel<T> {
    queue: Mutex<VecDeque<T>>,
    item_ready: Condvar,
}

impl<T> Channel<T> {
    fn new() -> Self {
        Channel {
            queue: Mutex::new(VecDeque::new()),
            item_ready: Condvar::new(),
        }
    }
    fn send(&self, message: T) {
        self.queue.lock().unwrap().push_back(message);
        self.item_ready.notify_one();
    }
    fn recv(&self) -> T {
        let mut guard = self.queue.lock().unwrap();
        loop {
            if let Some(message) = guard.pop_front() {
                return message;
            }
            guard = self.item_ready.wait(guard).unwrap();
        }
    }
}
fn main() {
    let channel: Channel<u32> = Channel::new();
    thread::scope(|s| {
        s.spawn(|| {
            sleep(Duration::from_secs(2));
            channel.send(23);
        });
        s.spawn(|| {
            let message = channel.recv();
            println!("{message}");
        });
    });
}
