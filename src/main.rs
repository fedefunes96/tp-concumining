use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;

static NTHREADS: i32 = 5;

fn main() {
    let mut miners = Vec::new();
    let mut receivers = Vec::new();
    let mut senders = Vec::new();

    //Canal de comunicacion del lider
    let (leader_tx, leader_rx): (Sender<i32>, Receiver<i32>) = mpsc::channel();

    //Creamos los canales de cada minero
    for _ in 0..NTHREADS {
        let (tx, rx): (Sender<i32>, Receiver<i32>) = mpsc::channel();

        senders.push(tx);
        receivers.push(rx);
    }

    for id in 0..NTHREADS {
        //Clonamos para poder transmitir a otros mineros
        let mut other_miners = senders.clone();

        //Clonamos para poder transmitir al lider
        let leader = leader_tx.clone();

        //Eliminamos a este minero de la lista de otros mineros
        other_miners.remove(id as usize);

        //Eliminamos el lado para recibir del canal de la lista, es unidireccional
        let rx = receivers.remove(0);

        let miner = thread::spawn(move || {
            //Enviamos un mensaje al siguiente minero en la lista
            other_miners[(id % (NTHREADS - 1)) as usize].send(id).unwrap();

            println!("Miner {} sent value {} to {}", id, id, (id + 1) % (NTHREADS));

            //Enviamos un mensaje al leader
            leader.send(id).unwrap();

            println!("Miner {} sent value {} to Leader(id = 500)", id, id);

            let mut val = rx.recv().unwrap();

            println!("Miner {} received message from {}", id, val);

            val = rx.recv().unwrap();

            println!("Miner {} received message from {}", id, val);
        });

        miners.push(miner);
    }

    for id in 0..NTHREADS {
        senders[id as usize].send(500).unwrap();
    }

    for _ in 0..NTHREADS {
        let val = leader_rx.recv().unwrap();

        println!("Leader received message from {}", val);
    }    
    
    for miner in miners {
        miner.join().expect("Miner panic");
    }
}
