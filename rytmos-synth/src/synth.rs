use rytmos_engrave::staff::Note;

use crate::commands::Command;

pub mod lpf;
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

    fn run_command(&mut self, command: Command) {
        match command {
            Command::Play(note, velocity, scale) => {
                let velocity: f32 = (velocity as f32 / 256.) * scale as f32;
                self.play(note, velocity);
            }
        }
    }
}
