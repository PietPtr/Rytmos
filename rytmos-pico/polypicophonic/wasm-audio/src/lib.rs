mod oscillator;

use js_sys::{Array, Float32Array, Math, Object};
// use log::Level;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub async fn audio_main() {
    // console_error_panic_hook::set_once();
    // console_log::init_with_level(Level::Debug).unwrap();
    // let params = Params {
    //     frequency: 3,
    //     volume: 100,
    // };
    // let mut osc = Oscillator::new(params);
    // log::info!("hey het werkt")
}

#[wasm_bindgen]
pub fn process(_inputs: Array, outputs: Array, _parameters: Object) {
    let output = outputs.get(0).unchecked_into::<Array>();

    for i in 0..output.length() {
        let channel = output.get(i).unchecked_into::<Float32Array>();

        for j in 0..channel.length() {
            channel.set_index(j, (Math::random() * 2.0 - 1.0) as f32);
        }
    }
}
