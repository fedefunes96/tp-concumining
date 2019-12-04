use std::collections::HashMap;
//use crate::logger::safe_writer::{SafeWriter};

pub struct MinersInfo {
    info: HashMap<usize, u32>
}

impl MinersInfo {
    pub fn new() -> MinersInfo {
        return MinersInfo { info: HashMap::new() }
    }

    pub fn insert(&mut self, key: usize, val: u32) {
        self.info.insert(key, val);
    }

    pub fn remove(&mut self, key: usize) {
        self.info.remove(&key);
    }

    pub fn get(&mut self, id: usize) -> u32 {
        match self.info.get(&id) {
            Some(val) => { return *val; }
            None => { self.info.insert(id, 0); return 0; }
        }
    }

    pub fn get_all(&self) -> HashMap<usize, u32> {
        return self.info.clone();
    }

    /*pub fn log_info(&self, logger: &mut SafeWriter) {
        for (id, value) in self.info.iter() {
            logger.write(format!("Miner {} has {} gold pips", id, value));
            println!("Miner {} has {} gold pips", id, value);
        }
    }*/

    pub fn get_worst_miners(&self) -> Vec<usize> {
        let mut worst_miners: Vec<usize> = Vec::new();

        let mut worst_pips = 9999;

        for (miner_id, gold_pips) in self.info.iter() {
            if *gold_pips < worst_pips {
                worst_pips = *gold_pips;
                worst_miners.clear();
                worst_miners.push(*miner_id);
            } else if *gold_pips == worst_pips {
                worst_miners.push(*miner_id);
            }
        }

        return worst_miners;
    }

    pub fn get_best_miners(&self) -> Vec<usize> {
        let mut best_miners: Vec<usize> = Vec::new();

        let mut best_pips = 0;

        for (miner_id, gold_pips) in self.info.iter() {
            if *gold_pips > best_pips {
                best_pips = *gold_pips;
                best_miners.clear();
                best_miners.push(*miner_id);
            } else if *gold_pips == best_pips {
                best_miners.push(*miner_id);
            }
        }

        return best_miners;
    }
}