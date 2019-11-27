#[path = "miner.rs"]
pub mod miner;

use crate::ipc::{Message, barrier::Barrier, Commands};


#[path = "miners_info.rs"]
pub mod miners_info;

use std::sync::mpsc::{Sender, Receiver};
use std::collections::HashMap;
use std::thread;
use std::time;

pub struct Leader {
    leader_rx: Receiver<Message>,
    senders: HashMap<usize, Sender<Message>>,
    condvar_listen: Barrier,
    condvar_transfer: Barrier
}

impl Leader {

    pub fn new(leader_rx: Receiver<Message>,
               senders: HashMap<usize, Sender<Message>>,
               condvar_listen: Barrier,
               condvar_transfer: Barrier) -> Leader {
        
        return Leader {
            leader_rx: leader_rx,
            senders: senders.clone(),
            condvar_listen: condvar_listen.clone(),
            condvar_transfer: condvar_transfer.clone()
        };
    }

    pub fn run(&mut self, 
               total_miners: usize,
               rounds: usize) {
        let leader_id = total_miners + 1;
        let mut miners_gold_pips = miners_info::MinersInfo::new();

        for miner_id in 0..total_miners {
            miners_gold_pips.insert(miner_id, 0);
        }

        for _ in 0..rounds {
            //Empieza una ronda nueva
            println!("Total miners {}", self.senders.len());
            
            if self.senders.len() == 1 {
                println!("Just one miner, finish");
                break;
            }
            //Ordenamos a los mineros a explorar
            self.give_orders(leader_id, Commands::EXPLORE);

            //Esperamos 2 segundos para dar la orden de regreso
            thread::sleep(time::Duration::from_millis(2000));

            //Ordenamos a los mineros a volver
            self.give_orders(leader_id, Commands::RETURN);

            self.condvar_listen.wait(self.senders.len() + 1);

            for _ in 0..(self.senders.len()) {
                let recv_msg = self.leader_rx.recv().unwrap();
                let gold_pips = recv_msg.extra.unwrap();
                miners_gold_pips.insert(recv_msg.id, gold_pips);

            }
            let worst_miners = miners_gold_pips.get_worst_miners();

            if worst_miners.len() == 1 {
                self.senders.remove(&worst_miners[0]);
                miners_gold_pips.remove(worst_miners[0]);
            }

            self.condvar_transfer.wait(self.senders.len() + 1);
        }  

        self.give_orders(leader_id, Commands::STOP);
    }

    fn give_orders(&self, leader_id: usize, command: Commands) {
        for miner_tx in self.senders.values() {
            let cmd = command.clone();

            let send_msg = Message {
                id: leader_id,
                cmd: cmd,
                extra: None
            };

            miner_tx.send(send_msg).unwrap();
        }    
    }
}