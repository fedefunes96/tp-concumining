pub struct Message {
    pub id: usize,
    pub cmd: Commands,
    pub extra: Option<u32>,
}

#[derive(Clone)]
pub enum Commands {
    EXPLORE,
    RETURN,
    STOP,
    LISTEN,
    TRANSFER
}
