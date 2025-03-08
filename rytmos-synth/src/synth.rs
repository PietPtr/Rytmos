use fixed::types::{I1F15, U4F4};
use rytmos_engrave::staff::Note;

use crate::commands::{Command, CommandMessage};

pub mod composed;
pub mod drum;
pub mod master;
pub mod metronome;
pub mod nothing;
pub mod sample;
pub mod samples;
pub mod sawtooth;
pub mod sine;
pub mod vibrato;

pub const SAMPLE_RATE: f32 = 24000.0;

pub trait Synth {
    type Settings;

    fn make(address: u32, settings: Self::Settings) -> Self
    where
        Self: Sized;
    fn configure(&mut self, settings: Self::Settings);
    fn play(&mut self, note: Note, velocity: U4F4);
    fn next(&mut self) -> I1F15;
    fn run_command(&mut self, command: Command);
    fn address(&self) -> u32;
}

fn run_play_command<S>(synth: &mut dyn Synth<Settings = S>, command: Command) {
    if command.address == synth.address() {
        if let CommandMessage::Play(note, velocity) = command.message {
            synth.play(note, velocity);
        }
    }
}
