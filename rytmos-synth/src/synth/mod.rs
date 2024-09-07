use rytmos_engrave::staff::Note;

pub mod lpf;
pub mod overtone;
pub mod sine;
pub mod vibrato;

pub const SAMPLE_RATE: f32 = 44100.0;

pub trait Synth {
    type Settings;

    fn configure(&mut self, settings: Self::Settings);
    fn play(&mut self, note: Note, velocity: f32);
    fn next(&mut self) -> i16;
}
