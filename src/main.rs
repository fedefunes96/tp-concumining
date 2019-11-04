use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;

static TOTAL_MINERS: i32 = 5;
static N_REGIONS: i32 = 5;

struct ExtraOpt {
    val: i32,
}

struct Message<'a> {
    id: usize,
    msg: &'a str,
    extra: Option<ExtraOpt>,
}

fn main() {
    let mut miners = Vec::new();
    let mut receivers = Vec::new();
    let mut senders = Vec::new();

    //Canal de comunicacion del lider
    let (leader_tx, leader_rx): (Sender<Message>, Receiver<Message>) = mpsc::channel();

    //Creamos los canales de cada minero
    for _ in 0..TOTAL_MINERS {
        let (tx, rx): (Sender<Message>, Receiver<Message>) = mpsc::channel();

        senders.push(tx);
        receivers.push(rx);
    }

    for id in 0..TOTAL_MINERS {
        //Clonamos para poder transmitir a otros mineros
        let mut other_miners = senders.clone();

        //Clonamos para poder transmitir al lider
        let leader = leader_tx.clone();

        //Eliminamos a este minero de la lista de otros mineros
        other_miners.remove(id as usize);

        //Eliminamos el lado para recibir del canal de la lista, es unidireccional
        let rx = receivers.remove(0);

        let miner = thread::spawn(move || {
            miner_loop(id, other_miners, rx, leader);
        });

        miners.push(miner);
    }

    leader_loop(leader_rx, senders);
  
    for miner in miners {
        miner.join().expect("Miner panic");
    }
}

fn miner_loop(
    id: i32,
    other_miners: Vec<Sender<Message>>,
    rx: Receiver<Message>,
    leader_tx: Sender<Message>) {

    let next_miner = (id % (TOTAL_MINERS - 1)) as usize;
            
    let send_msg = Message { id: id as usize, msg: "Pepitas", extra: None };
            
    println!("Miner {} sent message {} to {}", id, send_msg.msg, (id + 1) % (TOTAL_MINERS));

    other_miners[next_miner].send(send_msg).unwrap();

    let send_msg = Message { id: id as usize, msg: "Pepitas", extra: None };
            
    println!("Miner {} sent message {} to Leader", id, send_msg.msg);

    //Enviamos un mensaje al leader
    leader_tx.send(send_msg).unwrap();

    let mut recv_msg = rx.recv().unwrap();

    println!("Miner {} received message from {} saying {}", id, recv_msg.id, recv_msg.msg);

    recv_msg = rx.recv().unwrap();

    println!("Miner {} received message from {} saying {}", id, recv_msg.id, recv_msg.msg);
}

fn leader_loop(
    leader_rx: Receiver<Message>,
    senders: Vec<Sender<Message>>,
    ) {
    
    for id in 0..TOTAL_MINERS {
        let send_msg = Message { id: id as usize, msg: "Soy el lider", extra: None };
        senders[id as usize].send(send_msg).unwrap();
    }

    for _ in 0..TOTAL_MINERS {
        let recv_msg = leader_rx.recv().unwrap();

        println!("Leader received message from {} saying {}", recv_msg.id, recv_msg.msg);
    }    
}