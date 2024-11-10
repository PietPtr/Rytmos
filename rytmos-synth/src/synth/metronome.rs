use fixed::types::{I1F15, U8F8};
use log::info;
use rytmos_engrave::staff::{Accidental, Note};

use crate::commands::Command;

use super::{
    samples::{strong::STRONG_WAV, weak::WEAK_WAV},
    Synth,
};

pub struct MetronomeSynth {
    sample: usize,
    velocity: U8F8,
    play_sample: Option<Sample>,
}

enum Sample {
    Strong,
    Weak,
}

impl MetronomeSynth {
    pub fn new() -> Self {
        Self {
            sample: 0,
            velocity: 0.into(),
            play_sample: None,
        }
    }
}

impl Default for MetronomeSynth {
    fn default() -> Self {
        Self::new()
    }
}

impl Synth for MetronomeSynth {
    type Settings = ();

    fn configure(&mut self, _settings: Self::Settings) {}

    /// Ignores the frequency of the note and plays the metronome at the given velocity as amplifier
    /// with the set BPM.
    fn play(&mut self, note: Note, velocity: U8F8) {
        self.velocity = velocity;

        match note {
            Note::A(Accidental::Natural, _) => self.play_sample = Some(Sample::Strong),
            Note::B(Accidental::Natural, _) => self.play_sample = Some(Sample::Weak),
            _ => info!("unknown metronome note {note:?}"),
        }
    }

    // This cannot be synced to anything. Change it such that play actually plays the sample and decides on emphasis based on note.
    fn next(&mut self) -> I1F15 {
        let sample = match self.play_sample {
            Some(Sample::Strong) => {
                let sample_to_play = STRONG_WAV.get(self.sample);
                self.sample += 1;

                match sample_to_play {
                    Some(sample) => *sample,
                    None => {
                        self.play_sample = None; // Exhausted audio fragment.
                        0
                    }
                }
            }
            Some(Sample::Weak) => {
                let sample_to_play = WEAK_WAV.get(self.sample);
                self.sample += 1;

                match sample_to_play {
                    Some(sample) => *sample,
                    None => {
                        self.play_sample = None;
                        0
                    }
                }
            }
            None => 0,
        };

        // (sample as f32 * self.velocity) as i16
        todo!()
    }

    fn run_command(&mut self, command: Command) {
        super::run_play_command(self, command);
    }
}
