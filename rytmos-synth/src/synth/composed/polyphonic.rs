#[cfg(feature = "defmt")]
use defmt::warn;
use fixed::types::{I1F15, U4F4};
use heapless::Vec;
#[cfg(not(feature = "defmt"))]
use log::warn;
use rytmos_engrave::staff::Note;

/// TODO: move somewhere else
const fn ceil_log2(mut n: usize) -> u32 {
    let mut bits = 0;
    if n == 0 {
        return 0;
    }
    n -= 1;
    while n > 0 {
        n >>= 1;
        bits += 1;
    }
    bits
}

use crate::{
    commands::Command,
    synth::{run_play_command, Synth},
};

pub struct PolyphonicSynth<const P: usize, S: Synth> {
    address: u32,
    synths: [S; P],
    now_playing: [Option<Note>; P],
}

pub struct PolyphonicSynthSettings {}

impl<S: Synth, const P: usize> PolyphonicSynth<P, S> {
    const SHIFT: u32 = ceil_log2(P);
}

impl<S, const P: usize> Synth for PolyphonicSynth<P, S>
where
    S: Synth,
    S::Settings: Clone,
{
    type Settings = S::Settings;

    fn make(address: u32, settings: Self::Settings) -> Self
    where
        Self: Sized,
    {
        let mut synths: Vec<S, P> = Vec::new();
        for _ in 0..P {
            let synth = S::make(address, settings.clone());
            let _ = synths.push(synth);
        }

        let Ok(synths) = synths.into_array() else {
            panic!("Error in polyphonic synth initialization.");
        };

        Self {
            address,
            synths,
            now_playing: [None; P],
        }
    }

    fn configure(&mut self, _settings: Self::Settings) {
        todo!()
    }

    /// If the note is already played by a synth: play this new note on that synth
    /// If no synth is playing this yet, and one is available, play it there and update the tracking
    fn play(&mut self, note: Note, velocity: U4F4) {
        let synth_ids = self
            .now_playing
            .iter()
            .enumerate()
            .filter_map(|(index, note_opt)| match note_opt {
                Some(playing_note) => {
                    if *playing_note == note {
                        Some(index)
                    } else {
                        None
                    }
                }
                None => None,
            })
            .collect::<Vec<_, P>>();

        let synth_id = if let Some(&synth_id) = synth_ids.first() {
            // A synth is already playing this note, reuse it.
            Some(synth_id)
        } else {
            // No synth is playing this, check if there's an idle synth
            self.now_playing.iter().position(|&x| x.is_none())
        };

        if let Some(synth_id) = synth_id {
            self.synths[synth_id].play(note, velocity);

            if velocity.to_bits() == 0 {
                self.now_playing[synth_id] = None;
            } else {
                self.now_playing[synth_id] = Some(note);
            }
        } else {
            warn!(
                "No synth available to play {} {}",
                note.to_midi_code(),
                velocity.to_bits()
            )
        }
    }

    fn next(&mut self) -> I1F15 {
        let mut next = I1F15::ZERO;

        for synth in self.synths.iter_mut() {
            let sample = synth.next().to_bits();

            next += I1F15::from_bits(sample >> Self::SHIFT)
        }

        next
    }

    fn run_command(&mut self, command: Command) {
        run_play_command(self, command);
    }

    fn address(&self) -> u32 {
        self.address
    }
}
