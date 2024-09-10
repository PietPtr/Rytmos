use rytmos_engrave::staff::Note;

use crate::synth::SAMPLE_RATE;

use super::{
    samples::{strong::STRONG_WAV, weak::WEAK_WAV},
    Synth,
};

pub struct MetronomeSettings {
    pub bpm: usize,
    pub accent_one: bool,
}

pub struct Metronome {
    pub settings: MetronomeSettings,
    beat_count: usize,
    sample: usize,
    velocity: f32,
}

impl Metronome {
    pub fn new(settings: MetronomeSettings) -> Self {
        Self {
            settings,
            beat_count: 0,
            sample: 0,
            velocity: 0.,
        }
    }
}

impl Synth for Metronome {
    type Settings = MetronomeSettings;

    fn configure(&mut self, settings: Self::Settings) {
        self.settings = settings;
    }

    /// Ignores the frequency of the note and plays the metronome at the given velocity as amplifier
    /// with the set BPM.
    fn play(&mut self, _: Note, velocity: f32) {
        self.velocity = velocity;
    }

    fn next(&mut self) -> i16 {
        let next_beat = (60. / self.settings.bpm as f32) * SAMPLE_RATE;

        self.sample += 1;

        if self.sample == (next_beat as usize) {
            self.sample = 0;
            self.beat_count = (1 + self.beat_count) % 4;
        }

        if self.beat_count == 0 && self.settings.accent_one {
            *STRONG_WAV.get(self.sample).unwrap_or(&0)
        } else {
            *WEAK_WAV.get(self.sample).unwrap_or(&0)
        }
    }
}
