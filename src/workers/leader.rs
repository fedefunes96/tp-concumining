use crate::ipc::{Message, barrier::Barrier, Commands};
use crate::miners_info::{info::MinersInfo};

use std::sync::mpsc::{Sender, Receiver};
use std::collections::HashMap;
use std::thread;
use std::time;
use crate::logger::safe_writer::{SafeWriter};

pub struct Leader {
    leader_rx: Receiver<Message>,
    senders: HashMap<usize, Sender<Message>>,
    condvar_listen: Barrier,
    condvar_transfer: Barrier,
    logger: SafeWriter
}

impl Leader {

    pub fn new(leader_rx: Receiver<Message>,
               senders: HashMap<usize, Sender<Message>>,
               condvar_listen: Barrier,
               condvar_transfer: Barrier,
               logs: SafeWriter) -> Leader {
        
        return Leader {
            leader_rx: leader_rx,
            senders: senders.clone(),
            condvar_listen: condvar_listen.clone(),
            condvar_transfer: condvar_transfer.clone(),
            logger: logs.clone()
        };
    }

    pub fn run(&mut self, 
               total_miners: usize,
               rounds: usize) {
        let leader_id = total_miners + 1;
        let mut mined_gold_pips = MinersInfo::new();
        let mut total_gold_pips = MinersInfo::new();

        for miner_id in 0..total_miners {
            mined_gold_pips.insert(miner_id, 0);
            total_gold_pips.insert(miner_id, 0);
        }

        for _ in 0..rounds {
            //Empieza una ronda nueva
            //self.logger.write(format!("Total miners {}", self.senders.len()));
            //println!("Total miners {}", self.senders.len());
            
            if self.senders.len() == 1 {
                self.logger.write(format!("Just one miner, finish"));
                println!("Just one miner, finish");
                break;
            }
            self.check_all_pips(&total_gold_pips, &mined_gold_pips);
            //Ordenamos a los mineros a explorar
            self.give_orders(leader_id, Commands::EXPLORE);

            //Esperamos 2 segundos para dar la orden de regreso
            thread::sleep(time::Duration::from_millis(2000));

            //Ordenamos a los mineros a volver
            self.give_orders(leader_id, Commands::RETURN);

            self.condvar_listen.wait(self.senders.len() + 1);

            let mut new_mined_pips = MinersInfo::new();

            for _ in 0..(self.senders.len()) {
                let recv_msg = self.leader_rx.recv().unwrap();
                let gold_pips = recv_msg.extra.unwrap();
                new_mined_pips.insert(recv_msg.id, gold_pips);

                let actual_pips = total_gold_pips.get(recv_msg.id);
                total_gold_pips.insert(recv_msg.id, gold_pips + actual_pips);

                let mined_pips = mined_gold_pips.get(recv_msg.id);
                mined_gold_pips.insert(recv_msg.id, gold_pips + mined_pips);
            }
            let worst_miners = new_mined_pips.get_worst_miners();

            if worst_miners.len() == 1 {
                let best_miners = new_mined_pips.get_best_miners();

                self.senders.remove(&worst_miners[0]);
                //mined_gold_pips.remove(worst_miners[0]);

                //let tranfer_ammount = total_gold_pips.get(worst_miners[0]) / best_miners.len() as u32;

                //total_gold_pips.remove(worst_miners[0]);

                let pips_for_each = total_gold_pips.get(worst_miners[0]) / best_miners.len() as u32;
                let remainder = total_gold_pips.get(worst_miners[0]) % best_miners.len() as u32;

                total_gold_pips.insert(worst_miners[0], 0);

                let mut remainder_count = 0;

                for miner_id in best_miners {
                    let actual_pips = total_gold_pips.get(miner_id);
                    let mut final_pips = pips_for_each;

                    if remainder_count < remainder {
                        final_pips += 1;
                        remainder_count += 1;
                    }

                    total_gold_pips.insert(miner_id, actual_pips + final_pips);
                }
            }

            self.condvar_transfer.wait(self.senders.len() + 1);
        }  

        self.give_orders(leader_id, Commands::STOP);
        self.check_all_pips(&total_gold_pips, &mined_gold_pips);
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

    fn check_all_pips(&mut self, total_gold_pips: &MinersInfo, mined_gold_pips: &MinersInfo) {
        self.logger.write(format!("-----------"));
        self.logger.write(format!("Actual pips: Remaining Miners: {}", self.senders.len()));
        self.logger.write(format!("-----------"));
    
        let total_pips = total_gold_pips.get_all();
        let total_mined_pips = mined_gold_pips.get_all();

        for (id, pips) in total_pips.iter() {
            self.logger.write(
                format!(
                "Miner {} has {} gold pips | Mined {} gold pips"
                , id
                , pips
                , total_mined_pips[id])
            );
            println!("Miner {} has {} gold pips | Mined {} gold pips", id, pips, total_mined_pips[id]);
        }

        //total_gold_pips.log_info(&mut self.logger);

        self.logger.write(format!("-----------"));
    }
}
