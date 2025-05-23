use fixed::types::{I1F15, U4F4};
use log::info;
use rytmos_engrave::staff::{Accidental, Note};

use crate::commands::Command;

use super::{
    sample::{SampleSynth, SampleSynthSettings},
    samples, Synth,
};

// TODO: kinda useless as its just a subset of the drum synht?
pub struct MetronomeSynth {
    sample: usize,
    velocity: U4F4,
    address: u32,
    strong_synth: SampleSynth<samples::Strong>,
    weak_synth: SampleSynth<samples::Weak>,
}

impl MetronomeSynth {}

impl Synth for MetronomeSynth {
    type Settings = ();

    fn make(address: u32, _settings: Self::Settings) -> Self {
        Self {
            address,
            sample: 0,
            velocity: U4F4::from_num(0),
            strong_synth: SampleSynth::make(address, SampleSynthSettings {}),
            weak_synth: SampleSynth::make(address, SampleSynthSettings {}),
        }
    }

    fn configure(&mut self, _settings: Self::Settings) {}

    /// Ignores the frequency of the note and plays the metronome at the given velocity as amplifier
    /// with the set BPM.
    fn play(&mut self, note: Note, velocity: U4F4) {
        self.velocity = velocity;

        match note {
            Note::A(Accidental::Natural, _) => self.strong_synth.play(note, velocity),
            Note::B(Accidental::Natural, _) => self.weak_synth.play(note, velocity),
            _ => info!("unknown metronome note {note:?}"),
        }

        self.sample = 0
    }

    // TODO: This cannot be synced to anything. Change it such that play actually plays the sample and decides on emphasis based on note.
    fn next(&mut self) -> I1F15 {
        self.strong_synth
            .next()
            .saturating_add(self.weak_synth.next())
    }

    fn run_command(&mut self, command: Command) {
        super::run_play_command(self, command);
    }

    fn address(&self) -> u32 {
        self.address
    }

    fn freq(&mut self, freq: fixed::types::U12F4) {}

    // TODO: attack VS velocity naming convention?
    fn attack(&mut self, attack: U4F4) {
        self.velocity = attack
    }
}
