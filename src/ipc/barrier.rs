
use std::sync::{Arc, Mutex, Condvar};

#[derive(Clone)]
pub struct Barrier {
    last_value: usize,
    condvar: Arc<(Mutex<usize>, Condvar)>,
}

impl Barrier {
    pub fn new() -> Barrier {
        return Barrier {last_value: 0,
                        condvar: Arc::new((Mutex::new(0), Condvar::new()))};
    }

    pub fn wait(&mut self, value: usize) {
        let (lock, cvar) = &*self.condvar;
        let mut counter = lock.lock().unwrap();

        *counter += 1;

        if *counter == value + self.last_value {
            cvar.notify_all();
        }
        while *counter < value + self.last_value {
            counter = cvar.wait(counter).unwrap();
        }

        self.last_value += value;
    }
}