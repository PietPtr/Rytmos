use crate::clavier::KeyId;

pub struct IO<FIFO, CLAVIER> {
    pub fifo: FIFO,
    pub clavier: CLAVIER,
}

impl<FIFO, CLAVIER> IO<FIFO, CLAVIER>
where
    FIFO: Fifo,
    CLAVIER: ClavierPins,
{
    pub fn new(fifo: FIFO, clavier: CLAVIER) -> Self {
        Self { fifo, clavier }
    }
}

pub trait Fifo {
    fn write(&mut self, value: u32);
}

pub trait ClavierPins {
    fn read(&self, id: KeyId) -> bool;
}

// TODO: display and potentiometers
// TODO: ..audio/the synthesizer is technically an output too?
