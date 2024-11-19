use fixed::types::{I1F15, U4F4, U8F8};
use rytmos_engrave::{a, staff::Note};

use crate::commands::Command;

use super::{
    sine::{SineSynth, SineSynthSettings},
    Synth,
};

/// A sine synth with a wobbling frequency
pub struct VibratoSynth {
    address: u32,
    settings: VibratoSynthSettings,
    sine_synth: SineSynth,
    vibrato_synth: SineSynth,
    base_frequency: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct VibratoSynthSettings {
    pub sine_settings: SineSynthSettings,
    pub vibrato_frequency: f32,
    pub vibrato_strength: f32,
}

impl VibratoSynth {
    pub fn new(address: u32, settings: VibratoSynthSettings) -> Self {
        let mut vibrato_synth = SineSynth::new(
            address,
            SineSynthSettings {
                attack_gain: U4F4::from_num(1u8),
                initial_phase: 0.0,
                decay_per_second: 1.0,
            },
        );

        // vibrato_synth.play(a!(0), settings.vibrato_strength);
        vibrato_synth.set_frequency(settings.vibrato_frequency);

        Self {
            address,
            settings,
            sine_synth: SineSynth::new(address, settings.sine_settings),
            vibrato_synth,
            base_frequency: 0.,
        }
    }
}

impl Synth for VibratoSynth {
    type Settings = VibratoSynthSettings;

    fn configure(&mut self, settings: Self::Settings) {
        self.settings = settings
    }

    fn play(&mut self, note: Note, velocity: U4F4) {
        self.base_frequency = note.frequency();
        self.sine_synth.play(note, velocity);
    }

    fn next(&mut self) -> I1F15 {
        // let sine_freq = self.base_frequency + (self.vibrato_synth.next() as f32);

        // if sine_freq > 0. {
        //     self.sine_synth.set_frequency(sine_freq);
        // }

        // self.sine_synth.next()
        todo!()
    }

    fn run_command(&mut self, command: Command) {
        super::run_play_command(self, command);
    }

    fn address(&self) -> u32 {
        self.address
    }
}
