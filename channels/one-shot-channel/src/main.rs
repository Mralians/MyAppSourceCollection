use std::sync::{Condvar, Mutex};
use std::thread;
use std::time::Duration;

struct Channel<T> {
    ready: Condvar,
    messsage: Mutex<Option<T>>,
}
impl<T> Channel<T> {
    fn new() -> Self {
        Self {
            ready: Condvar::new(),
            messsage: Mutex::new(Option::None),
        }
    }
    fn send(&self, messsage: T) {
        let mut guard = self.messsage.lock().unwrap();
        *guard = Some(messsage);
        self.ready.notify_one();
    }
    fn receive(&self) -> T {
        let mut guard = self.messsage.lock().unwrap();
        let message = loop {
            if let Some(messsage) = guard.take() {
                break messsage;
            } else {
                guard = self.ready.wait(guard).unwrap();
            }
        };
        message
    }
}
unsafe impl<T> Sync for Channel<T> where T: Send {}
fn main() {
    let channel = Channel::new();

    thread::scope(|s| {
        s.spawn(|| {
            thread::sleep(Duration::from_secs(2));
            channel.send("Hello!");
        });
        s.spawn(|| {
            let message = channel.receive();
            println!("{message}");
        });
    });
}
