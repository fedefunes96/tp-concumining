#[path = "miner.rs"]
pub mod miner;

use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::sync::{Arc, Mutex, Condvar};
use std::collections::HashMap;
use std::thread;
use std::time;

#[derive(Clone)]
enum Commands {
    EXPLORE,
    RETURN,
    STOP,
    LISTEN,
    TRANSFER
}

struct Message {
    id: usize,
    cmd: Commands,
    extra: Option<i32>,
}

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

        //Canal de comunicacion del lider
        let (leader_tx, leader_rx): (Sender<Message>, Receiver<Message>) = mpsc::channel();

        //Creamos los canales de cada minero
        for id in 0..self.total_miners {
            let (tx, rx): (Sender<Message>, Receiver<Message>) = mpsc::channel();

            senders.insert(id, tx);
            receivers.push(rx);
        }

        let condvar_return = Arc::new((Mutex::new(0), Condvar::new()));
        let condvar_listen = Arc::new((Mutex::new(0), Condvar::new()));
        let condvar_transfer = Arc::new((Mutex::new(0), Condvar::new()));

        for id in 0..self.total_miners {
            //Clonamos para poder transmitir a otros mineros
            let mut other_miners = senders.clone();
            
            //Clonamos las Condvar
            let condvar_miner_return = condvar_return.clone();
            let condvar_miner_listen = condvar_listen.clone();
            let condvar_miner_transfer = condvar_transfer.clone();
            //Clonamos para poder transmitir al lider
            let leader = leader_tx.clone();

            //Eliminamos a este minero de la lista de otros mineros
            other_miners.remove(&id);

            //Eliminamos el lado para recibir del canal de la lista, es unidireccional
            let rx = receivers.remove(0);
            let miner = thread::spawn(move || {
                miner_loop(
                    id, 
                    &mut other_miners,
                    rx,
                    leader,
                    condvar_miner_return,
                    condvar_miner_listen,
                    condvar_miner_transfer
                );
            });
            
            miners.push(miner);

        }

        self.leader_loop(leader_rx, &mut senders, condvar_listen, condvar_transfer);
    
        for miner in miners {
            miner.join().expect("Miner panic");
        }
    }

    fn leader_loop(&self,
        leader_rx: Receiver<Message>,
        senders: &mut HashMap<usize, Sender<Message>>,
        condvar_listen: Arc<(Mutex<usize>, Condvar)>,
        condvar_transfer: Arc<(Mutex<usize>, Condvar)>
    ) {
        let leader_id = self.total_miners + 1;
        let mut miners_gold_pips = HashMap::new();
        let mut last_all_ready1 = 0;
        let mut last_all_ready2 = 0;

        for miner_id in 0..self.total_miners {
            miners_gold_pips.insert(miner_id, 0);
        }

        for _ in 0..self.rounds {
            //Empieza una ronda nueva
            println!("Total miners {}", senders.len());
            
            if senders.len() == 1 {
                println!("Just one miner, finish");
                break;
            }
            //Ordenamos a los mineros a explorar
            self.give_orders(leader_id, &senders, Commands::EXPLORE);

            //Esperamos 2 segundos para dar la orden de regreso
            thread::sleep(time::Duration::from_millis(2000));

            //Ordenamos a los mineros a volver
            self.give_orders(leader_id, &senders, Commands::RETURN);

            wait(senders.len() + 1 + last_all_ready1, &condvar_listen);

            last_all_ready1 += senders.len() + 1;

            for _ in 0..(senders.len()) {
                let recv_msg = leader_rx.recv().unwrap();

                let gold_pips = miners_gold_pips.entry(recv_msg.id).or_insert(0);

                *gold_pips = recv_msg.extra.unwrap();  
            }
            let worst_miners = get_worst_miners(&miners_gold_pips);

            if worst_miners.len() == 1 {
                senders.remove(&worst_miners[0]);
                miners_gold_pips.remove(&worst_miners[0]);
            }

            wait(senders.len() + 1 + last_all_ready2, &condvar_transfer);

            last_all_ready2 += senders.len() + 1;
        }  

        self.give_orders(leader_id, &senders, Commands::STOP);
    }

    fn give_orders(&self, leader_id: usize, senders: &HashMap<usize, Sender<Message>>, command: Commands) {
        for miner_tx in senders.values() {
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

fn get_worst_miners(miners_gold_pips: &HashMap<usize, i32>) -> Vec<usize> {
    let mut worst_miners: Vec<usize> = Vec::new();

    let mut worst_pips = 9999;

    for (miner_id, gold_pips) in miners_gold_pips.iter() {
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

fn get_best_miners(miners_gold_pips: &HashMap<usize, i32>) -> Vec<usize> {
    let mut best_miners: Vec<usize> = Vec::new();

    let mut best_pips = 0;

    for (miner_id, gold_pips) in miners_gold_pips.iter() {
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
fn miner_loop(
    id: usize,
    other_miners: &mut HashMap<usize, Sender<Message>>,
    rx: Receiver<Message>,
    leader_tx: Sender<Message>,
    condvar_return: Arc<(Mutex<usize>, Condvar)>,
    condvar_listen: Arc<(Mutex<usize>, Condvar)>,
    condvar_transfer: Arc<(Mutex<usize>, Condvar)>
) {
    let total_miners : usize = other_miners.len() + 1;
    let mut counter_told = 0;
    let mut total_gold_pips = 0;
    let mut last_miners_ready = 0;
    let mut last_all_ready_1 = 0;
    let mut last_all_ready_2 = 0;
    let mut miners_gold_pips = HashMap::new();

    for miner_id in 0..total_miners {
        miners_gold_pips.insert(miner_id, 0);
    }

    loop {
        //Esperamos recibir un mensaje para operar
        let recv_msg = rx.recv().unwrap();

        match recv_msg.cmd {
            Commands::EXPLORE => {
                println!("Miner {} was sent to explore a region", id);
                
                let gold_pips = miners_gold_pips.entry(id).or_insert(0);

                *gold_pips = miner::explore();

                total_gold_pips += miners_gold_pips[&id];
            },
            Commands::RETURN => {
                println!("Miner {} returned with {} gold pips", id, miners_gold_pips[&id]);

                //Esperamos a que los demas mineros vuelvan
                wait(other_miners.len() + 1 + last_miners_ready, &condvar_return);
                last_miners_ready += other_miners.len() + 1;

                for miner in other_miners.values() {
                    let send_msg = Message {
                        id: id,
                        cmd: Commands::LISTEN,
                        extra: Some(miners_gold_pips[&id].clone())
                    };

                    miner.send(send_msg).unwrap();
                }

                let send_msg = Message {
                    id: id,
                    cmd: Commands::LISTEN,
                    extra: Some(miners_gold_pips[&id].clone())
                };

                leader_tx.send(send_msg).unwrap();
            },
            Commands::LISTEN => {
                println!("Miner {} was told by Miner {} that this mined {} gold pips", id, recv_msg.id, recv_msg.extra.unwrap());
                
                let gold_pips = miners_gold_pips.entry(recv_msg.id).or_insert(0);

                *gold_pips = recv_msg.extra.unwrap();
                                
                counter_told += 1;

                if counter_told == other_miners.len() {
                    counter_told = 0;
                    //Esperamos a que todos sepan la cantidad de
                    //pepitas de oro que minó cada minero
                    wait(other_miners.len() + 2 + last_all_ready_1, &condvar_listen);

                    last_all_ready_1 += other_miners.len() + 2;

                    let worst_miners = get_worst_miners(&miners_gold_pips);
                    let best_miners = get_best_miners(&miners_gold_pips);

                    if worst_miners.len() > 1 {
                        println!("More than 2 miners are the worst");

                        wait(other_miners.len() + 2 + last_all_ready_2, &condvar_transfer);

                        last_all_ready_2 += other_miners.len() + 2;
                        continue;
                    }

                    //Veo si soy el peor minero
                    if worst_miners.contains(&id) {
                        let tranfer_ammount = total_gold_pips / best_miners.len() as i32;

                        for miner_id in best_miners {
                            let send_msg = Message {
                                id: id,
                                cmd: Commands::TRANSFER,
                                extra: Some(tranfer_ammount.clone())
                            };

                            other_miners[&miner_id].send(send_msg).unwrap();                            
                        }
                        println!("Miner {} retired", id);   
                        break;
                    } else {
                        other_miners.remove(&worst_miners[0]);
                        miners_gold_pips.remove(&worst_miners[0]);
                    }

                    //Veo si soy uno de los mejores mineros
                    if best_miners.contains(&id) {
                        continue;
                    }
                    wait(other_miners.len() + 2 + last_all_ready_2, &condvar_transfer);

                    last_all_ready_2 += other_miners.len() + 2;
                }
            },     
            Commands::TRANSFER => {
                println!("Miner {} received {} pips from Miner {}", id, recv_msg.extra.unwrap(), recv_msg.id);

                total_gold_pips += recv_msg.extra.unwrap();
                wait(other_miners.len() + 2 + last_all_ready_2, &condvar_transfer);

                last_all_ready_2 += other_miners.len() + 2;             
            },
            Commands::STOP => {
                break;
            }
        }

        //break;
    }
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