use fixed::types::{I1F15, U4F4};
use rytmos_engrave::staff::Note;

use crate::commands::{Command, CommandMessage};

pub mod exponential_decay;
pub mod linear_decay;
pub mod lpf;

pub trait Effect {
    type Settings;

    fn make(address: u32, settings: Self::Settings) -> Self
    where
        Self: Sized;

    fn configure(&mut self, settings: Self::Settings);

    /// Effects modify a single input stream into a new output stream
    /// at the same sample rate.
    fn next(&mut self, input: I1F15) -> I1F15;

    /// In some cases filters need to know when and what note was attacked to
    /// properly process. Giving the Effect the same address and routing the same
    /// commands to it as the synth it is effecting on achieves this effect.
    fn play(&mut self, note: Note, velocity: U4F4);

    /// An effect is free to implement commands however it sees fit,
    /// So names may not necessarily match exact functionality.
    fn run_command(&mut self, command: Command);

    fn address(&self) -> u32;
}

fn run_play_command<S>(effect: &mut dyn Effect<Settings = S>, command: Command) {
    if command.address == effect.address() {
        if let CommandMessage::Play(note, velocity) = command.message {
            effect.play(note, velocity);
        }
    }
}
