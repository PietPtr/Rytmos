mod oscillator;

use log::Level;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub async fn audio_main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(Level::Debug).unwrap();
    // let params = Params {
    //     frequency: 3,
    //     volume: 100,
    // };
    // let mut osc = Oscillator::new(params);
    log::info!("hey het werkt")
}
