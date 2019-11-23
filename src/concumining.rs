#[path = "miner.rs"]
pub mod miner;

#[path = "ipc/mod.rs"]
pub mod ipc;

#[path = "miners_info.rs"]
pub mod miners_info;

use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::sync::{Arc, Mutex, Condvar};
use std::collections::HashMap;
use std::thread;
use std::time;


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
        let (leader_tx, leader_rx): (Sender<miner::ipc::Message>, Receiver<miner::ipc::Message>) = mpsc::channel();

        //Creamos los canales de cada minero
        for id in 0..self.total_miners {
            let (tx, rx): (Sender<miner::ipc::Message>, Receiver<miner::ipc::Message>) = mpsc::channel();

            senders.insert(id, tx);
            receivers.push(rx);
        }

        let condvar_return = Arc::new((Mutex::new(0), Condvar::new()));
        let condvar_listen = Arc::new((Mutex::new(0), Condvar::new()));
        let condvar_transfer = Arc::new((Mutex::new(0), Condvar::new()));

        for id in 0..self.total_miners {
            //Eliminamos el lado para recibir del canal de la lista, es unidireccional
            let rx = receivers.remove(0);
            //Clonamos para poder transmitir a otros mineros
            let mut other_miners = senders.clone();
            
            //Eliminamos a este minero de la lista de otros mineros
            other_miners.remove(&id);

            let mut the_miner = miner::Miner::new(id, &mut other_miners, rx,
                leader_tx.clone(), 
                condvar_return.clone(), 
                condvar_listen.clone(), 
                condvar_transfer.clone());

            let miner = thread::spawn(move || {
                the_miner.run()
            });
            
            miners.push(miner);

        }

        self.leader_loop(leader_rx, &mut senders, condvar_listen, condvar_transfer);
    
        for miner in miners {
            miner.join().expect("Miner panic");
        }
    }

    fn leader_loop(&self,
        leader_rx: Receiver<miner::ipc::Message>,
        senders: &mut HashMap<usize, Sender<miner::ipc::Message>>,
        condvar_listen: Arc<(Mutex<usize>, Condvar)>,
        condvar_transfer: Arc<(Mutex<usize>, Condvar)>
    ) {
        let leader_id = self.total_miners + 1;
        let mut miners_gold_pips = miners_info::MinersInfo::new();
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
            self.give_orders(leader_id, &senders, miner::ipc::Commands::EXPLORE);

            //Esperamos 2 segundos para dar la orden de regreso
            thread::sleep(time::Duration::from_millis(2000));

            //Ordenamos a los mineros a volver
            self.give_orders(leader_id, &senders, miner::ipc::Commands::RETURN);

            wait(senders.len() + 1 + last_all_ready1, &condvar_listen);

            last_all_ready1 += senders.len() + 1;

            for _ in 0..(senders.len()) {
                let recv_msg = leader_rx.recv().unwrap();
                let gold_pips = recv_msg.extra.unwrap();
                miners_gold_pips.insert(recv_msg.id, gold_pips);

            }
            let worst_miners = miners_gold_pips.get_worst_miners();

            if worst_miners.len() == 1 {
                senders.remove(&worst_miners[0]);
                miners_gold_pips.remove(worst_miners[0]);
            }

            wait(senders.len() + 1 + last_all_ready2, &condvar_transfer);

            last_all_ready2 += senders.len() + 1;
        }  

        self.give_orders(leader_id, &senders, miner::ipc::Commands::STOP);
    }

    fn give_orders(&self, leader_id: usize, senders: &HashMap<usize, Sender<miner::ipc::Message>>, command: miner::ipc::Commands) {
        for miner_tx in senders.values() {
            let cmd = command.clone();

            let send_msg = miner::ipc::Message {
                id: leader_id,
                cmd: cmd,
                extra: None
            };

            miner_tx.send(send_msg).unwrap();
        }    
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