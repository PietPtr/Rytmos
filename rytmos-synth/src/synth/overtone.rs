use fixed::types::{I1F15, U4F4};
use rytmos_engrave::staff::Note;

use crate::commands::Command;

use super::Synth;

/// Synthesizer that overlays a single synth N times in its integer overtone series.
pub struct OvertoneSynth<S: Synth, const N: usize> {
    address: u32,
    settings: OvertoneSynthSettings<N>,
    synths: [S; N],
}

pub struct OvertoneSynthSettings<const N: usize> {}

impl<S: Synth, const N: usize> OvertoneSynth<S, N> {
    pub fn new(address: u32, settings: OvertoneSynthSettings<N>, synths: [S; N]) -> Self {
        Self {
            address,
            settings,
            synths,
        }
    }
}

impl<S: Synth, const N: usize> Synth for OvertoneSynth<S, N> {
    type Settings = OvertoneSynthSettings<N>;

    fn configure(&mut self, settings: Self::Settings) {
        self.settings = settings
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
        super::run_play_command(self, command);
    }

    fn address(&self) -> u32 {
        self.address
    }
}
