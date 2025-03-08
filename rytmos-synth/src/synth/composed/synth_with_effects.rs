use fixed::types::{I1F15, U4F4};
use rytmos_engrave::staff::Note;

use crate::{commands::Command, effect::Effect, synth::Synth};

pub struct SynthWithEffect<S: Synth, E: Effect> {
    synth: S,
    effect: E,
}

impl<S: Synth, E: Effect> SynthWithEffect<S, E> {
    pub fn new(synth: S, effect: E) -> Self {
        Self { synth, effect }
    }
}

pub struct SynthWithEffectSettings<S: Synth, E: Effect> {
    pub synth: S::Settings,
    pub effect: E::Settings,
}

impl<S: Synth, E: Effect> Clone for SynthWithEffectSettings<S, E>
where
    S::Settings: Clone,
    E::Settings: Clone,
{
    fn clone(&self) -> Self {
        Self {
            synth: self.synth.clone(),
            effect: self.effect.clone(),
        }
    }
}

impl<S, E> Default for SynthWithEffectSettings<S, E>
where
    S: Synth,
    E: Effect,
    S::Settings: Default,
    E::Settings: Default,
{
    fn default() -> Self {
        Self {
            synth: S::Settings::default(),
            effect: E::Settings::default(),
        }
    }
}

impl<S: Synth, E: Effect> Synth for SynthWithEffect<S, E> {
    type Settings = SynthWithEffectSettings<S, E>;

    fn make(address: u32, settings: Self::Settings) -> Self {
        SynthWithEffect::<S, E>::new(
            S::make(address, settings.synth),
            E::make(address, settings.effect),
        )
    }

    fn configure(&mut self, settings: Self::Settings) {
        self.synth.configure(settings.synth);
        self.effect.configure(settings.effect);
    }

    fn play(&mut self, note: Note, velocity: U4F4) {
        self.synth.play(note, velocity);
        self.effect.play(note, velocity);
    }

    fn next(&mut self) -> I1F15 {
        self.effect.next(self.synth.next())
    }

    fn run_command(&mut self, command: Command) {
        self.synth.run_command(command);
        self.effect.run_command(command);
    }

    // TODO: fun idea, let's see how it goes
    fn address(&self) -> u32 {
        self.synth.address() | self.effect.address()
    }
}
