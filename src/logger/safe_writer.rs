use std::fs::File;
use std::io::prelude::*;
use std::sync::{Arc, Mutex};

pub struct SafeWriter {
    fd: File,
    m: Arc<Mutex<i32>>,
}

impl SafeWriter {
    pub fn new(file: &str) -> SafeWriter {
        return SafeWriter {fd: File::create(file).unwrap(),
                m: Arc::new(Mutex::new(0))};
    }

    pub fn write(&mut self, buf: String) {
        let _guard = self.m.lock().unwrap();

        self.fd.write_all(buf.as_bytes()).unwrap();
        self.fd.write_all(b"\n").unwrap();

        //self.fd.sync_all().unwrap();
    }
}

impl Clone for SafeWriter {
    fn clone(&self) -> SafeWriter {
        return SafeWriter {fd: self.fd.try_clone().unwrap(),
                m: self.m.clone()};
    }
}
