use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;

static NTHREADS: i32 = 5;

struct ExtraMessage {
    val: i32,
}

struct Message<'a> {
    id: usize,
    msg: &'a str,
    extra: Option<ExtraMessage>,
}

fn main() {
    let mut miners = Vec::new();
    let mut receivers = Vec::new();
    let mut senders = Vec::new();

    //Canal de comunicacion del lider
    let (leader_tx, leader_rx): (Sender<Message>, Receiver<Message>) = mpsc::channel();

    //Creamos los canales de cada minero
    for _ in 0..NTHREADS {
        let (tx, rx): (Sender<Message>, Receiver<Message>) = mpsc::channel();

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
            let next_miner = (id % (NTHREADS - 1)) as usize;
            
            let send_msg = Message { id: id as usize, msg: "Pepitas", extra: None };
            
            println!("Miner {} sent message {} to {}", id, send_msg.msg, (id + 1) % (NTHREADS));

            other_miners[next_miner].send(send_msg).unwrap();

            let send_msg = Message { id: id as usize, msg: "Pepitas", extra: None };
            
            println!("Miner {} sent message {} to Leader", id, send_msg.msg);

            //Enviamos un mensaje al leader
            leader.send(send_msg).unwrap();

            let mut recv_msg = rx.recv().unwrap();

            println!("Miner {} received message from {} saying {}", id, recv_msg.id, recv_msg.msg);

            recv_msg = rx.recv().unwrap();

            println!("Miner {} received message from {} saying {}", id, recv_msg.id, recv_msg.msg);
        });

        miners.push(miner);
    }

    for id in 0..NTHREADS {
        let send_msg = Message { id: id as usize, msg: "Soy el lider", extra: None };
        senders[id as usize].send(send_msg).unwrap();
    }

    for _ in 0..NTHREADS {
        let recv_msg = leader_rx.recv().unwrap();

        println!("Leader received message from {} saying {}", recv_msg.id, recv_msg.msg);
    }    
    
    for miner in miners {
        miner.join().expect("Miner panic");
    }
}
