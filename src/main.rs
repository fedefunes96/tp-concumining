mod miner;

use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time;

static TOTAL_MINERS: usize = 5;
static N_REGIONS: usize = 5;
static LEADER_ID: usize = TOTAL_MINERS + 1;

#[derive(Clone)]
enum Commands {
    EXPLORE,
    RETURN,
    STOP,
    TELL
}

struct Message {
    id: usize,
    cmd: Commands,
    extra: Option<i32>,
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

    let barrier_return = Arc::new(Barrier::new(TOTAL_MINERS));
    let barrier_tell = Arc::new(Barrier::new(TOTAL_MINERS + 1));

    for id in 0..TOTAL_MINERS {
        //Clonamos para poder transmitir a otros mineros
        let mut other_miners = senders.clone();
        
        //Clonamos las barreras
        let barrier_miner_return = barrier_return.clone();
        let barrier_miner_tell = barrier_tell.clone();

        //Clonamos para poder transmitir al lider
        let leader = leader_tx.clone();

        //Eliminamos a este minero de la lista de otros mineros
        other_miners.remove(id);

        //Eliminamos el lado para recibir del canal de la lista, es unidireccional
        let rx = receivers.remove(0);

        let miner = thread::spawn(move || {
            miner_loop(
                id, 
                other_miners,
                rx,
                leader,
                barrier_miner_return,
                barrier_miner_tell
            );
        });

        miners.push(miner);
    }

    leader_loop(leader_rx, senders, barrier_tell);
  
    for miner in miners {
        miner.join().expect("Miner panic");
    }
}

fn miner_loop(
    id: usize,
    other_miners: Vec<Sender<Message>>,
    rx: Receiver<Message>,
    leader_tx: Sender<Message>,
    barrier_return: Arc<Barrier>,
    barrier_tell: Arc<Barrier>
    ) {

    let mut gold_pips = 0;
    let mut counter_told = 0;

    loop {
        //Esperamos recibir un mensaje para operar
        let recv_msg = rx.recv().unwrap();

        match recv_msg.cmd {
            Commands::EXPLORE => {
                println!("Miner {} was sent to explore a region", id);

                gold_pips = miner::explore();
            },
            Commands::RETURN => {
                println!("Miner {} returned with {} gold pips", id, gold_pips);

                //Esperamos a que los demas mineros vuelvan
                barrier_return.wait();
                
                for miner in &other_miners {
                    let send_msg = Message {
                        id: id,
                        cmd: Commands::TELL,
                        extra: Some(gold_pips.clone())
                    };

                    miner.send(send_msg).unwrap();
                }
            },
            Commands::TELL => {
                println!("Miner {} was told by Miner {} that this mined {} gold pips", id, recv_msg.id, recv_msg.extra.unwrap());

                counter_told += 1;

                if counter_told == TOTAL_MINERS - 1 {
                    //Esperamos a que todos (incluyendo el lider) sepan la cantidad de
                    //pepitas de oro que minó cada minero
                    barrier_tell.wait();           

                    counter_told = 0;
                }
            },
            Commands::STOP => {
                break;
            }
        }

        //break;
    }
}

fn leader_loop(
    leader_rx: Receiver<Message>,
    senders: Vec<Sender<Message>>,
    barrier_tell: Arc<Barrier>
    ) {

    for _ in 0..N_REGIONS {
        //Empieza una ronda nueva

        //Ordenamos a los mineros a explorar
        give_orders(&senders, Commands::EXPLORE);

        //Esperamos 2 segundos para dar la orden de regreso
        thread::sleep(time::Duration::from_millis(2000));

        //Ordenamos a los mineros a volver
        give_orders(&senders, Commands::RETURN);

        barrier_tell.wait();
    }  

    give_orders(&senders, Commands::STOP);
}

fn give_orders(senders: &Vec<Sender<Message>>, command: Commands) {
    for miner_tx in senders {
        let cmd = command.clone();

        let send_msg = Message {
            id: LEADER_ID,
            cmd: cmd,
            extra: None
        };

        miner_tx.send(send_msg).unwrap();
    }    
}
