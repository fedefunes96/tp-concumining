extern crate rand;
extern crate rand_distr;

use crate::miners_info::{info::MinersInfo};

static MAX_GOLD_PIPS: f64 = 20.0;

use rand_distr::{Distribution, Beta};

use crate::ipc::{barrier::Barrier, Message, Commands};
use crate::logger::safe_writer::{SafeWriter};

use std::sync::mpsc::{Sender, Receiver};
use std::collections::HashMap;


pub struct Miner {
    listen: Barrier,
    _return: Barrier,
    transfer: Barrier,
    id: usize,
    recv_channel: Receiver<Message>,
    leader: Sender<Message>,
    other_miners: HashMap<usize, Sender<Message>>,
    logger: SafeWriter
}

impl Miner {
    pub fn new(id: usize,
           other_miners: &mut HashMap<usize, Sender<Message>>,
           rx: Receiver<Message>,
           leader_tx: Sender<Message>,
           condvar_return: Barrier,
           condvar_listen: Barrier,
           condvar_transfer: Barrier,
           logs: SafeWriter) -> Miner {
        
        return Miner {
            listen: condvar_listen.clone(),
            _return: condvar_return.clone(),
            transfer: condvar_transfer.clone(),
            id: id,
            recv_channel: rx,
            leader: leader_tx,
            other_miners: other_miners.clone(),
            logger: logs.clone()
        }
    }

    pub fn run(&mut self) {
        let total_miners : usize = self.other_miners.len() + 1;
        let mut counter_told = 0;
        let mut total_gold_pips = 0;
        let mut miners_gold_pips = MinersInfo::new();

        for miner_id in 0..total_miners {
            miners_gold_pips.insert(miner_id, 0);
        }

        loop {
            //Esperamos recibir un mensaje para operar
            let recv_msg = self.recv_channel.recv().unwrap();

            match recv_msg.cmd {
                Commands::EXPLORE => {
                    self.miner_explore(&mut miners_gold_pips, &mut total_gold_pips)
                },
                Commands::RETURN => {
                    self.miner_return(&mut miners_gold_pips);
                },
                Commands::LISTEN => {
                    let miner_continue = self.miner_listen(recv_msg.id, recv_msg.extra.unwrap(), &mut counter_told, total_gold_pips, &mut miners_gold_pips);

                    if !miner_continue {
                        break;
                    }
                },     
                Commands::TRANSFER => {
                    self.miner_transfer(recv_msg.id, recv_msg.extra.unwrap(), &mut total_gold_pips);
                },
                Commands::STOP => {
                    break;
                }
            }

            //break;
        }
    }

    fn explore(&self) -> u32 {
        let mut rng = rand::thread_rng();

        let beta = Beta::new(2.0, 5.0).unwrap();

        return (beta.sample(&mut rng) * MAX_GOLD_PIPS) as u32;
    }

    fn miner_explore(&mut self, miners_gold_pips: &mut MinersInfo, total_gold_pips: &mut u32) {
        self.logger.write(format!("Miner {} was sent to explore a region", self.id));
        println!("Miner {} was sent to explore a region", self.id);
                    
        let gold_pips = self.explore();
        miners_gold_pips.insert(self.id, gold_pips);
        *total_gold_pips += gold_pips;        
    }

    fn miner_return(&mut self, miners_gold_pips: &mut MinersInfo) {
        self.logger.write(format!("Miner {} returned with {} gold pips", self.id, miners_gold_pips.get(self.id)));
        println!("Miner {} returned with {} gold pips", self.id, miners_gold_pips.get(self.id));

        //Esperamos a que los demas mineros vuelvan
        self._return.wait(self.other_miners.len() + 1);

        self.send_command_all(Commands::LISTEN, Some(miners_gold_pips.get(self.id).clone()));

        self.send_command_leader(Commands::LISTEN, Some(miners_gold_pips.get(self.id).clone()));
    }

    fn miner_listen(&mut self, who: usize, pips: u32, counter_told: &mut usize, total_gold_pips: u32, miners_gold_pips: &mut MinersInfo) -> bool {
        self.logger.write(format!("Miner {} was told by Miner {} that this mined {} gold pips", self.id, who, pips));
        println!("Miner {} was told by Miner {} that this mined {} gold pips", self.id, who, pips);
                    
        miners_gold_pips.insert(who, pips);
        *counter_told += 1;

        if *counter_told == self.other_miners.len() {
            *counter_told = 0;
            //Esperamos a que todos sepan la cantidad de
            //pepitas de oro que minó cada minero
            self.listen.wait(self.other_miners.len() + 2);

            let worst_miners = miners_gold_pips.get_worst_miners();
            let best_miners = miners_gold_pips.get_best_miners();

            if worst_miners.len() > 1 {
                self.logger.write(format!("More than 2 miners are the worst"));
                println!("More than 2 miners are the worst");

                self.transfer.wait(self.other_miners.len() + 2);
                return true;
            }

            //Veo si soy el peor minero
            if worst_miners.contains(&self.id) {
                self.send_gold_pips(best_miners, total_gold_pips);

                self.logger.write(format!("Miner {} retired", self.id));
                println!("Miner {} retired", self.id);   
                return false;
            } else {
                self.other_miners.remove(&worst_miners[0]);
                miners_gold_pips.remove(worst_miners[0]);
            }

            //Veo si soy uno de los mejores mineros
            if best_miners.contains(&self.id) {
                return true;
            }
            self.transfer.wait(self.other_miners.len() + 2);
        }

        return true;
    }

    fn miner_transfer(&mut self, who: usize, receive: u32, total_gold_pips: &mut u32) {
        self.logger.write(format!("Miner {} received {} pips from Miner {}", self.id, receive, who));
        println!("Miner {} received {} pips from Miner {}", self.id, receive, who);

        *total_gold_pips += receive;
        self.transfer.wait(self.other_miners.len() + 2);
    }

    fn send_command_all(&mut self, command: Commands, extra_data: Option<u32>) {
        for miner in self.other_miners.values() {
            let send_msg = Message {
                id: self.id,
                cmd: command.clone(),
                extra: extra_data
            };

            miner.send(send_msg).unwrap();
        }
    }

    fn send_gold_pips(&mut self, best_miners: Vec<usize>, total_pips: u32) {
        let pips_for_each = total_pips / best_miners.len() as u32;
        let remainder = total_pips % best_miners.len() as u32;

        let mut remainder_count = 0;
        for miner in best_miners {
            let mut final_pips = pips_for_each;
            if remainder_count < remainder {
                final_pips += 1;
                remainder_count += 1;
            }

            let send_msg = Message {
                id: self.id,
                cmd: Commands::TRANSFER,
                extra: Some(final_pips)
            };

            self.other_miners[&miner].send(send_msg).unwrap();
        }
    }

    fn send_command_leader(&mut self,command: Commands, extra_data: Option<u32>) {
        let send_msg = Message {
            id: self.id,
            cmd: command.clone(),
            extra: extra_data
        };

        self.leader.send(send_msg).unwrap();        
    }
}
