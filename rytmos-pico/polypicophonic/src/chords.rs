use heapless::Vec;
use rytmos_engrave::staff::Note;
use rytmos_synth::commands::CommandMessage;

use crate::clavier::NoteEvent;

pub enum ChordQuality {
    Major,
    Minor,
}

#[allow(dead_code)]
pub enum ChordConstruction {
    DiatonicUp,
    InvertToWithinOctave,
}

pub fn root_to_bass_register(messages: &mut Vec<CommandMessage, 4>) {
    let Some(root_message) = messages.first_mut() else {
        return;
    };

    let CommandMessage::Play(root, _) = root_message else {
        return;
    };

    root.map_octave(|_| 2);
}

pub fn add_chord(
    messages: &mut Vec<CommandMessage, 4>,
    quality: ChordQuality,
    construction: ChordConstruction,
) {
    let (root, velocity) = {
        let Some(root_message) = messages.first() else {
            return;
        };

        let CommandMessage::Play(root, velocity) = root_message else {
            return;
        };

        (*root, *velocity)
    };

    let (third, fifth) = match construction {
        ChordConstruction::DiatonicUp => diatonic_up(root, quality),
        ChordConstruction::InvertToWithinOctave => invert_to_within_octave(root, quality),
    };

    let _ = messages.push(CommandMessage::Play(third, velocity));
    let _ = messages.push(CommandMessage::Play(fifth, velocity));
}

pub fn diatonic_up(root: Note, quality: ChordQuality) -> (Note, Note) {
    let third = match quality {
        ChordQuality::Major => Note::from_u8_flat(root.to_midi_code() + 4),
        ChordQuality::Minor => Note::from_u8_flat(root.to_midi_code() + 3),
    };

    let fifth = Note::from_u8_flat(root.to_midi_code() + 7);

    (third, fifth)
}

pub fn invert_to_within_octave(root: Note, quality: ChordQuality) -> (Note, Note) {
    let (mut third, mut fifth) = diatonic_up(root, quality);

    if third.octave() > root.octave() {
        third.map_octave(|o| o - 1);
        fifth.map_octave(|o| o - 1);
    }

    if fifth.octave() > root.octave() {
        fifth.map_octave(|o| o - 1);
    }

    (third, fifth)
}
