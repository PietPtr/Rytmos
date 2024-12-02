use derivative::Derivative;
use fixed::types::{I1F15, U4F4};

use rytmos_engrave::staff::Note;

use crate::commands::Command;

use super::{run_play_command, Effect};

#[derive(Derivative)]
#[derivative(Default)]
#[derive(Debug, Clone, Copy)]
pub struct LinearDecaySettings {
    #[derivative(Default(value = "I1F15::from_num(0.001)"))]
    pub decay: I1F15,
    #[derivative(Default(value = "32"))]
    pub decay_every: usize,
}

pub struct LinearDecay {
    settings: LinearDecaySettings,
    decay_counter: usize,
    amplitude: I1F15,
    address: u32,
}

impl LinearDecay {
    pub fn new(address: u32, settings: LinearDecaySettings) -> Self {
        Self {
            settings,
            decay_counter: 0,
            amplitude: I1F15::from_bits(0),
            address,
        }
    }
}

impl Effect for LinearDecay {
    type Settings = LinearDecaySettings;

    fn next(&mut self, input: I1F15) -> I1F15 {
        self.decay_counter += 1;

        if self.decay_counter == self.settings.decay_every {
            self.amplitude = (self.amplitude - self.settings.decay).max(I1F15::from_bits(0));
            self.decay_counter = 0;
        }

        input * self.amplitude
    }

    fn play(&mut self, _note: Note, _velocity: U4F4) {
        self.amplitude = I1F15::MAX;
    }

    fn run_command(&mut self, command: Command) {
        run_play_command(self, command);
    }

    fn address(&self) -> u32 {
        self.address
    }
}
