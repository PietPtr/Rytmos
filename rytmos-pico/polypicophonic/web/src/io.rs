use core::assert_eq;

use dioxus::signals::{Readable, Signal};
use polypicophonic::io;
use web_sys::{wasm_bindgen::JsValue, AudioWorkletNode};

pub struct WebFifo {
    node: Signal<Option<AudioWorkletNode>>,
}

impl WebFifo {
    pub fn new(node: Signal<Option<AudioWorkletNode>>) -> Self {
        Self { node }
    }
}

impl io::Fifo for WebFifo {
    fn write(&mut self, value: u32) {
        let binding = self.node.read();
        let node = binding.as_ref().unwrap();
        node.port()
            .unwrap()
            .post_message(&JsValue::from_f64(value as f64))
            .unwrap();
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
