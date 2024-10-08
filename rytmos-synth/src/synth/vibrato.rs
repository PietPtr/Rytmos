use rytmos_engrave::{a, staff::Note};

use crate::commands::Command;

use super::{
    sine::{SineSynth, SineSynthSettings},
    Synth,
};

/// A sine synth with a wobbling frequency
pub struct VibratoSynth {
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
    pub fn new(settings: VibratoSynthSettings) -> Self {
        let mut vibrato_synth = SineSynth::new(SineSynthSettings {
            attack_gain: 1.0,
            initial_phase: 0.0,
            decay_per_second: 1.0,
        });

        vibrato_synth.play(a!(0), settings.vibrato_strength);
        vibrato_synth.set_frequency(settings.vibrato_frequency);

        Self {
            settings,
            sine_synth: SineSynth::new(settings.sine_settings),
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

    fn play(&mut self, note: Note, velocity: f32) {
        self.base_frequency = note.frequency();
        self.sine_synth.play(note, velocity);
    }

    fn next(&mut self) -> i16 {
        let sine_freq = self.base_frequency + (self.vibrato_synth.next() as f32);

        if sine_freq > 0. {
            self.sine_synth.set_frequency(sine_freq);
        }

        self.sine_synth.next()
    }

    fn run_command(&mut self, command: Command) {
        super::run_play_command(self, command);
    }
}
