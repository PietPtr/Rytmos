use rytmos_engrave::staff::Note;

use crate::commands::Command;

pub mod lpf;
pub mod master;
pub mod metronome;
pub mod overtone;
pub mod samples;
pub mod sine;
pub mod vibrato;

pub const SAMPLE_RATE: f32 = 44100.0;

pub trait Synth {
    type Settings;

    fn configure(&mut self, settings: Self::Settings);
    fn play(&mut self, note: Note, velocity: f32);
    fn next(&mut self) -> i16;
    fn run_command(&mut self, command: Command);
}

fn run_play_command<S>(synth: &mut dyn Synth<Settings = S>, command: Command) {
    if let Command::Play(note, velocity, scale) = command {
        let velocity: f32 = (velocity as f32 / 256.) * scale as f32;
        synth.play(note, velocity);
    }
}
