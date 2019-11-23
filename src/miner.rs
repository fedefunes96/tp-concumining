extern crate rand;
extern crate rand_distr;

#[path = "miners_info.rs"]
pub mod miners_info;

static MAX_GOLD_PIPS: f64 = 20.0;

use rand_distr::{Distribution, Beta};

#[path = "ipc/mod.rs"]
pub mod ipc;

use std::sync::mpsc::{Sender, Receiver};
use std::sync::{Arc, Mutex, Condvar};
use std::collections::HashMap;


pub struct Miner {
    listen: Arc<(Mutex<usize>, Condvar)>,
    _return: Arc<(Mutex<usize>, Condvar)>,
    transfer: Arc<(Mutex<usize>, Condvar)>,
    id: usize,
    recv_channel: Receiver<ipc::Message>,
    leader: Sender<ipc::Message>,
    other_miners: HashMap<usize, Sender<ipc::Message>>
}

impl Miner {
    pub fn new(id: usize,
           other_miners: &mut HashMap<usize, Sender<ipc::Message>>,
           rx: Receiver<ipc::Message>,
           leader_tx: Sender<ipc::Message>,
           condvar_return: Arc<(Mutex<usize>, Condvar)>,
           condvar_listen: Arc<(Mutex<usize>, Condvar)>,
           condvar_transfer: Arc<(Mutex<usize>, Condvar)>) -> Miner {
        
        return Miner {
            listen: condvar_listen.clone(),
            _return: condvar_return.clone(),
            transfer: condvar_transfer.clone(),
            id: id,
            recv_channel: rx,
            leader: leader_tx,
            other_miners: other_miners.clone()
        }
    }

    pub fn run(&mut self) {
        let total_miners : usize = self.other_miners.len() + 1;
        let mut counter_told = 0;
        let mut total_gold_pips = 0;
        let mut last_miners_ready = 0;
        let mut last_all_ready_1 = 0;
        let mut last_all_ready_2 = 0;
        let mut miners_gold_pips = miners_info::MinersInfo::new();

        for miner_id in 0..total_miners {
            miners_gold_pips.insert(miner_id, 0);
        }

        loop {
            //Esperamos recibir un mensaje para operar
            let recv_msg = self.recv_channel.recv().unwrap();

            match recv_msg.cmd {
                ipc::Commands::EXPLORE => {
                    println!("Miner {} was sent to explore a region", self.id);
                    

                    let gold_pips = explore();
                    miners_gold_pips.insert(self.id, gold_pips);
                    total_gold_pips += gold_pips;
                },
                ipc::Commands::RETURN => {
                    println!("Miner {} returned with {} gold pips", self.id, miners_gold_pips.get(self.id));

                    //Esperamos a que los demas mineros vuelvan
                    wait(self.other_miners.len() + 1 + last_miners_ready, &self._return);
                    last_miners_ready += self.other_miners.len() + 1;

                    for miner in self.other_miners.values() {
                        let send_msg = ipc::Message {
                            id: self.id,
                            cmd: ipc::Commands::LISTEN,
                            extra: Some(miners_gold_pips.get(self.id).clone())
                        };

                        miner.send(send_msg).unwrap();
                    }

                    let send_msg = ipc::Message {
                        id: self.id,
                        cmd: ipc::Commands::LISTEN,
                        extra: Some(miners_gold_pips.get(self.id).clone())
                    };

                    self.leader.send(send_msg).unwrap();
                },
                ipc::Commands::LISTEN => {
                    println!("Miner {} was told by Miner {} that this mined {} gold pips", self.id, recv_msg.id, recv_msg.extra.unwrap());
                    
                    let gold_pips = recv_msg.extra.unwrap();
                    miners_gold_pips.insert(recv_msg.id, gold_pips);
                    counter_told += 1;

                    if counter_told == self.other_miners.len() {
                        counter_told = 0;
                        //Esperamos a que todos sepan la cantidad de
                        //pepitas de oro que minó cada minero
                        wait(self.other_miners.len() + 2 + last_all_ready_1, &self.listen);

                        last_all_ready_1 += self.other_miners.len() + 2;

                        let worst_miners = miners_gold_pips.get_worst_miners();
                        let best_miners = miners_gold_pips.get_best_miners();

                        if worst_miners.len() > 1 {
                            println!("More than 2 miners are the worst");

                            wait(self.other_miners.len() + 2 + last_all_ready_2, &self.transfer);

                            last_all_ready_2 += self.other_miners.len() + 2;
                            continue;
                        }

                        //Veo si soy el peor minero
                        if worst_miners.contains(&self.id) {
                            let tranfer_ammount = total_gold_pips / best_miners.len() as u32;

                            for miner_id in best_miners {
                                let send_msg = ipc::Message {
                                    id: self.id,
                                    cmd: ipc::Commands::TRANSFER,
                                    extra: Some(tranfer_ammount.clone())
                                };

                                self.other_miners[&miner_id].send(send_msg).unwrap();                            
                            }
                            println!("Miner {} retired", self.id);   
                            break;
                        } else {
                            self.other_miners.remove(&worst_miners[0]);
                            miners_gold_pips.remove(worst_miners[0]);
                        }

                        //Veo si soy uno de los mejores mineros
                        if best_miners.contains(&self.id) {
                            continue;
                        }
                        wait(self.other_miners.len() + 2 + last_all_ready_2, &self.transfer);

                        last_all_ready_2 += self.other_miners.len() + 2;
                    }
                },     
                ipc::Commands::TRANSFER => {
                    println!("Miner {} received {} pips from Miner {}", self.id, recv_msg.extra.unwrap(), recv_msg.id);

                    total_gold_pips += recv_msg.extra.unwrap();
                    wait(self.other_miners.len() + 2 + last_all_ready_2, &self.transfer);

                    last_all_ready_2 += self.other_miners.len() + 2;             
                },
                ipc::Commands::STOP => {
                    break;
                }
            }

            //break;
        }
    }
}

pub fn explore() -> u32 {
    let mut rng = rand::thread_rng();

    let beta = Beta::new(2.0, 5.0).unwrap();

    return (beta.sample(&mut rng) * MAX_GOLD_PIPS) as u32;
}


fn wait(value: usize,
    condvar: &Arc<(Mutex<usize>, Condvar)>
) {
    let (lock, cvar) = &**condvar;
    let mut counter = lock.lock().unwrap();

    *counter += 1;

    if *counter == value {
        cvar.notify_all();
    }
    while *counter < value {
        counter = cvar.wait(counter).unwrap();
    }
}