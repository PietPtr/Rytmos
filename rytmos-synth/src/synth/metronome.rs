use fixed::types::{I1F15, U4F4};
use log::info;
use rytmos_engrave::staff::{Accidental, Note};

use crate::commands::Command;

use super::{
    samples::{strong::STRONG_WAV_24000, weak::WEAK_WAV_24000},
    Synth,
};

pub struct MetronomeSynth {
    sample: usize,
    velocity: U4F4,
    play_sample: Option<Sample>,
    address: u32,
}

enum Sample {
    Strong,
    Weak,
}

impl MetronomeSynth {}

impl Synth for MetronomeSynth {
    type Settings = ();

    fn make(address: u32, settings: Self::Settings) -> Self {
        Self {
            address,
            sample: 0,
            velocity: U4F4::from_num(0),
            play_sample: None,
        }
    }

    fn configure(&mut self, _settings: Self::Settings) {}

    /// Ignores the frequency of the note and plays the metronome at the given velocity as amplifier
    /// with the set BPM.
    fn play(&mut self, note: Note, velocity: U4F4) {
        self.velocity = velocity;

        match note {
            Note::A(Accidental::Natural, _) => self.play_sample = Some(Sample::Strong),
            Note::B(Accidental::Natural, _) => self.play_sample = Some(Sample::Weak),
            _ => info!("unknown metronome note {note:?}"),
        }

        self.sample = 0
    }

    // This cannot be synced to anything. Change it such that play actually plays the sample and decides on emphasis based on note.
    fn next(&mut self) -> I1F15 {
        let sample = match self.play_sample {
            Some(Sample::Strong) => {
                let sample_to_play = STRONG_WAV_24000.get(self.sample);
                self.sample += 1;

                match sample_to_play {
                    Some(sample) => *sample,
                    None => {
                        self.play_sample = None; // Exhausted audio fragment.
                        I1F15::from_num(0)
                    }
                }
            }
            Some(Sample::Weak) => {
                let sample_to_play = WEAK_WAV_24000.get(self.sample);
                self.sample += 1;

                match sample_to_play {
                    Some(sample) => *sample,
                    None => {
                        self.play_sample = None;
                        I1F15::from_num(0)
                    }
                }
            }
            None => I1F15::from_num(0),
        };

        sample
    }

    fn run_command(&mut self, command: Command) {
        super::run_play_command(self, command);
    }

    fn address(&self) -> u32 {
        self.address
    }
}
