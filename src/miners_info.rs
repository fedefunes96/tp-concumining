use std::collections::HashMap;

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