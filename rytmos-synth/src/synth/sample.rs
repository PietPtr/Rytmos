use fixed::types::{I1F15, U4F4};
use rytmos_engrave::staff::Note;

use crate::commands::Command;

use super::Synth;

pub struct SampleSynth<'a> {
    place_in_sample: usize,
    velocity: U4F4,
    address: u32,
    is_playing: bool,
    settings: SampleSynthSettings<'a>,
}

pub struct SampleSynthSettings<'a> {
    pub sample: &'a [I1F15],
}

impl<'a> Synth for SampleSynth<'a> {
    type Settings = SampleSynthSettings<'a>;

    fn make(address: u32, settings: Self::Settings) -> Self {
        Self {
            address,
            place_in_sample: 0,
            velocity: U4F4::from_num(0),
            is_playing: false,
            settings,
        }
    }

    fn configure(&mut self, _settings: Self::Settings) {}

    fn play(&mut self, _note: Note, velocity: U4F4) {
        self.velocity = velocity;
        self.place_in_sample = 0;
        self.is_playing = true;
    }

    fn next(&mut self) -> I1F15 {
        let sample = {
            let sample_to_play = self.settings.sample.get(self.place_in_sample);
            self.place_in_sample += 1;

            match sample_to_play {
                Some(sample) => *sample,
                None => {
                    self.is_playing = false; // Exhausted audio fragment.
                    I1F15::from_num(0)
                }
            }
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
