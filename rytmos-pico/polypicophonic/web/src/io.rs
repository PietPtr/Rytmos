use core::assert_eq;

use dioxus::signals::{Readable, Signal};
use polypicophonic::io;

pub struct WebFifo {}

impl io::Fifo for WebFifo {
    fn write(&mut self, value: u32) {
        tracing::info!("Sending command {value}");
    }
}

pub struct WebKeys {
    key_signals: Vec<Signal<bool>>,
}

impl WebKeys {
    pub fn new(key_signals: Vec<Signal<bool>>) -> Self {
        assert_eq!(key_signals.len(), 16);
        Self { key_signals }
    }
}

impl io::ClavierPins for WebKeys {
    fn read(&self, id: polypicophonic::clavier::KeyId) -> bool {
        *self.key_signals.get(id as usize).unwrap().read()
    }
}
