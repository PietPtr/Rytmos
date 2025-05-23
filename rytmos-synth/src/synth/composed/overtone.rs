use fixed::types::{I1F15, U4F4};
use rytmos_engrave::staff::Note;

use crate::{commands::Command, synth::Synth};

/// Synthesizer that overlays a single synth N times in its integer overtone series.
pub struct OvertoneSynth<S: Synth, const N: usize> {
    address: u32,
    synths: [S; N],
}

pub struct OvertoneSynthSettings<S: Synth, const N: usize> {
    pub synths: [S::Settings; N],
}

impl<S: Synth, const N: usize> Synth for OvertoneSynth<S, N> {
    type Settings = OvertoneSynthSettings<S, N>;

    fn make(address: u32, settings: Self::Settings) -> Self {
        let synths = settings.synths.map(|settings| S::make(address, settings));
        Self { address, synths }
    }

    fn configure(&mut self, settings: Self::Settings) {
        for (synth, settings) in self.synths.iter_mut().zip(settings.synths) {
            synth.configure(settings);
        }
    }

    fn play(&mut self, mut note: Note, velocity: U4F4) {
        for (i, synth) in self.synths.iter_mut().enumerate() {
            synth.play(note.map_octave(|n| n + i as i32), velocity)
        }
    }

    fn next(&mut self) -> I1F15 {
        self.synths
            .iter_mut()
            .fold(I1F15::from_bits(0), |acc, synth| {
                acc.saturating_add(synth.next())
            })
    }

    fn run_command(&mut self, command: Command) {
        crate::synth::run_play_command(self, command);
    }

    fn address(&self) -> u32 {
        self.address
    }

    fn freq(&mut self, freq: fixed::types::U12F4) {
        todo!()
    }

    fn attack(&mut self, attack: U4F4) {
        todo!()
    }
}
