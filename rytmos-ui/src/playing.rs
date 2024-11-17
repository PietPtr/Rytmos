use fixed::types::U8F8;
use rytmos_engrave::{c, staff::Note};
use rytmos_synth::commands::Command;

use crate::interface::PlayingButtons;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayAction {
    Mute,
    PlayOpen,
    PlayFret1,
    PlayFret2,
    PlayFret3,
    PlayFret4,
}

/// Contains the logic to turn changes in the fretting and plucking button state into an action.
/// How the action maps to notes is defined by whatever initializes this.
pub struct FrettingAndPlucking {
    last_state: Option<PlayingButtons>,
}

impl FrettingAndPlucking {
    pub fn new() -> Self {
        Self { last_state: None }
    }

    pub fn action(&mut self, state: PlayingButtons) -> Option<PlayAction> {
        if self.last_state.is_none() {
            self.last_state = Some(state);
            return None;
        }

        let last_state = self.last_state.unwrap();

        // if any note is fretted and any of the plucks move from 0 to 1
        // fastest plucker: 0 1 | 1 0 | 0 1
        // realistic:  111111000000000001111110000
        //             000000000111110000000111111
        // plays:      x        x       x   x
        let attack = matches!(
            (last_state.plucking_buttons, state.plucking_buttons),
            ([false, _], [true, _]) | ([_, false], [_, true])
        );

        // a mute occurs the moment the player releases all fretting buttons
        //   f1 111111
        //   f2     111111111
        //   f3             11111       1111
        //   f4                                  1111  11111
        // mute:                 x          x        x      x
        let mute = last_state.fretting_buttons.iter().any(|&x| x)
            && state.fretting_buttons.iter().all(|&x| !x);

        self.last_state = Some(state);

        // TODO: Hammer-on and pull-off (which should be a different action so synths can accomodate)

        match (mute, attack) {
            (true, _) => Some(PlayAction::Mute),
            (false, true) => Some(match state.fretting_buttons {
                [_, _, _, true] => PlayAction::PlayFret4,
                [_, _, true, _] => PlayAction::PlayFret3,
                [_, true, _, _] => PlayAction::PlayFret2,
                [true, _, _, _] => PlayAction::PlayFret1,
                [false, false, false, false] => PlayAction::PlayOpen,
            }),
            _ => None,
        }
    }
}

impl Default for FrettingAndPlucking {
    fn default() -> Self {
        Self::new()
    }
}

pub trait ActionToCommand {
    fn translate(&self, play_action: PlayAction) -> Option<Command>;
}

// Simple translation that maps the fret buttons to the first four frets of the given "string" (note).
pub struct ChromaticActionToCommand {
    notes: [Note; 5],
}

impl ChromaticActionToCommand {
    pub fn new(string_tuning: Note) -> Self {
        let code = string_tuning.to_midi_code();
        Self {
            notes: [
                Note::from_u8_sharp(code),
                Note::from_u8_sharp(code + 1),
                Note::from_u8_sharp(code + 2),
                Note::from_u8_sharp(code + 3),
                Note::from_u8_sharp(code + 4),
            ],
        }
    }
}

impl ActionToCommand for ChromaticActionToCommand {
    fn translate(&self, play_action: PlayAction) -> Option<Command> {
        let note = match play_action {
            PlayAction::Mute => c!(0),
            PlayAction::PlayOpen => self.notes[0],
            PlayAction::PlayFret1 => self.notes[1],
            PlayAction::PlayFret2 => self.notes[2],
            PlayAction::PlayFret3 => self.notes[3],
            PlayAction::PlayFret4 => self.notes[4],
        };

        let velocity = if play_action == PlayAction::Mute {
            U8F8::from_num(0.)
        } else {
            U8F8::from_num(1.)
        };

        Some(Command::Play(note, velocity))
    }
}
