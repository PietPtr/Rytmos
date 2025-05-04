pub struct Oscillator {
    params: Params,
    accumulator: u32,
}

impl Oscillator {
    pub fn new(params: Params) -> Self {
        Self {
            params,
            accumulator: 0,
        }
    }
}

impl Oscillator {
    pub fn process(&mut self, output: &mut [f32]) -> bool {
        // This method is called in the audio process thread.
        // All imports are set, so host functionality available in worklets
        // (for example, logging) can be used:
        // `web_sys::console::log_1(&JsValue::from(output.len()));`
        // Note that currently TextEncoder and TextDecoder are stubs, so passing
        // strings may not work in this thread.
        for a in output {
            let frequency = self.params.frequency;
            let volume = self.params.volume;
            self.accumulator += u32::from(frequency);
            *a = (self.accumulator as f32 / 512.).sin() * (volume as f32 / 100.);
        }
        true
    }

    pub fn set_params(&mut self, params: Params) {
        self.params = params
    }
}

#[derive(Default)]
pub struct Params {
    pub frequency: u8,
    pub volume: u8,
}
