use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;

static TOTAL_MINERS: usize = 5;
static N_REGIONS: usize = 5;
static LEADER_ID: usize = TOTAL_MINERS + 1;

enum Commands {
    EXPLORE,
}

struct ExtraOpt {
    val: i32,
}

struct Message {
    id: usize,
    cmd: Commands,
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
        other_miners.remove(id);

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
    id: usize,
    other_miners: Vec<Sender<Message>>,
    rx: Receiver<Message>,
    leader_tx: Sender<Message>) {

    loop {
        //Esperamos recibir un mensaje para operar
        let mut recv_msg = rx.recv().unwrap();

        match recv_msg.cmd {
            Commands::EXPLORE => {
                println!("Miner {} was sent to explore a region", id);
            }
        }

        break;
    }
}

fn leader_loop(
    leader_rx: Receiver<Message>,
    senders: Vec<Sender<Message>>,
    ) {

    for _ in 0..N_REGIONS {
        //Empieza una ronda nueva
        for miner_tx in &senders {
            let send_msg = Message {
                id: LEADER_ID,
                cmd: Commands::EXPLORE,
                extra: None
            };

            miner_tx.send(send_msg).unwrap();
        }

        break;
    }  
}
