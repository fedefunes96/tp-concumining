use crate::workers::{miner::Miner, leader::Leader};
use crate::ipc::{Message, barrier::Barrier};
use crate::logger::safe_writer::{SafeWriter};

use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::collections::HashMap;
use std::thread;

const LOG_FILE:&str = "/tmp/logs_concumining";

pub struct Concumining {
    pub total_miners: usize,
    pub rounds: usize,
}

impl Concumining {
    pub fn new(miners: usize, rounds: usize) -> Concumining  {
        return Concumining {total_miners: miners, rounds: rounds};
    }

    pub fn start(&self) {
        let mut miners = Vec::new();
        let mut receivers = Vec::new();
        let mut senders = HashMap::new();

        let mut logger = SafeWriter::new(LOG_FILE);

        logger.write(format!("------------------"));
        logger.write(format!("Starting log debug"));
        logger.write(format!("------------------"));
        //Canal de comunicacion del lider
        let (leader_tx, leader_rx): (Sender<Message>, Receiver<Message>) = mpsc::channel();

        //Creamos los canales de cada minero
        for id in 0..self.total_miners {
            let (tx, rx): (Sender<Message>, Receiver<Message>) = mpsc::channel();

            senders.insert(id, tx);
            receivers.push(rx);
        }

        let condvar_return = Barrier::new();
        let condvar_listen = Barrier::new();
        let condvar_transfer = Barrier::new();

        for id in 0..self.total_miners {
            //Eliminamos el lado para recibir del canal de la lista, es unidireccional
            let rx = receivers.remove(0);
            //Clonamos para poder transmitir a otros mineros
            let mut other_miners = senders.clone();
            
            //Eliminamos a este minero de la lista de otros mineros
            other_miners.remove(&id);

            let mut the_miner = Miner::new(id, &mut other_miners, rx,
                leader_tx.clone(), 
                condvar_return.clone(), 
                condvar_listen.clone(), 
                condvar_transfer.clone(),
                logger.clone());

            let miner = thread::spawn(move || {
                the_miner.run()
            });
            
            miners.push(miner);

        }

        let mut leader = Leader::new(
            leader_rx,
        senders, condvar_listen.clone(), condvar_transfer.clone(),
        logger.clone());
        leader.run(self.total_miners, self.rounds);
    
        for miner in miners {
            miner.join().expect("Miner panic");
        }
    }
}