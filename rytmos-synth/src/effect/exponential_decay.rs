use derivative::Derivative;
use fixed::types::{I1F15, U4F4};

use rytmos_engrave::staff::Note;

use crate::commands::Command;

use super::{run_play_command, Effect};

/// Applies a decay on the input signal by setting the amplitude to 1.0 on an
/// attack, then decaying that amplitude by multiplying by the `decay` value every
/// `decay_every` samples.
#[derive(Derivative)]
#[derivative(Default)]
#[derive(Debug, Clone, Copy)]
pub struct ExponentialDecaySettings {
    #[derivative(Default(value = "I1F15::from_num(0.99)"))]
    pub decay: I1F15,
    #[derivative(Default(value = "32"))]
    pub decay_every: usize,
}

pub struct ExponentialDecay {
    settings: ExponentialDecaySettings,
    decay_counter: usize,
    amplitude: I1F15,
    address: u32,
}

impl Effect for ExponentialDecay {
    type Settings = ExponentialDecaySettings;

    fn make(address: u32, settings: Self::Settings) -> Self {
        Self {
            settings,
            decay_counter: 0,
            amplitude: I1F15::from_bits(0),
            address,
        }
    }

    fn configure(&mut self, settings: Self::Settings) {
        self.settings = settings;
    }

    fn next(&mut self, input: I1F15) -> I1F15 {
        self.decay_counter += 1;

        if self.decay_counter == self.settings.decay_every {
            self.amplitude *= self.settings.decay;
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
