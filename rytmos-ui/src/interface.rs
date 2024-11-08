use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};
use heapless::Vec;
use rytmos_engrave::{
    a,
    staff::{Clef, Music, Staff, StaffElement},
};
use rytmos_scribe::sixteen_switches::{MeasureState, RhythmDefinition, SwitchState};
use rytmos_synth::commands::Command;

use crate::{
    // play_analysis::PlayAnalysis,
    bare_menu::BareMenu,
    playing::{ActionToCommand, ChromaticActionToCommand, FrettingAndPlucking},
};

pub const DISPLAY_SIZE: Size = Size::new(128, 64);

#[derive(Default, Debug, Copy, Clone)]
pub struct IOState {
    /// The sixteen tri-state toggle switches for defining rhythms
    pub toggle_switches: [SwitchState; 16],
    /// Four buttons for the fretting hand and two buttons for string plucking
    pub playing_buttons: PlayingButtons,
    /// Four buttons below the screen for menu navigation
    pub menu_buttons: [bool; 4],
}

#[derive(Default, Debug, Copy, Clone)]
pub struct PlayingButtons {
    pub fretting_buttons: [bool; 4],
    pub plucking_buttons: [bool; 2],
}

/// The top level of all interfaces in the device.
///
/// MVP:
/// - master synth containing an overtone synth with some constant settings
/// - connect fretting and plucking buttons to master synth
/// - bare menu at the bottom of the screen
pub struct Interface {
    // Gadgets, drawables
    staff: Staff,
    // analysis: PlayAnalysis,
    states: MeasureState,
    menu: BareMenu,

    // IO related
    io_state: IOState, // TODO: really necessary to store?
    fretting_and_plucking: FrettingAndPlucking,
    action_to_command: ChromaticActionToCommand,

    // Logic state, maybe should be empty as this is state for inside gadgets?
    // ringing: bool,
    music: Vec<Music, 16>,
}

impl Default for Interface {
    fn default() -> Self {
        Self::new()
    }
}

impl Interface {
    pub fn new() -> Self {
        Self {
            staff: Staff::new(DISPLAY_SIZE.width, Point::new(0, 0)),
            // analysis: PlayAnalysis::new(RhythmDefinition::default()),
            states: MeasureState::default(),
            menu: BareMenu::new(),
            io_state: IOState::default(),
            fretting_and_plucking: FrettingAndPlucking::default(),
            action_to_command: ChromaticActionToCommand::new(a!(1)),
            // ringing: false,
            music: Vec::new(),
        }
    }

    pub fn draw<D>(&mut self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = BinaryColor>,
    {
        Rectangle::new(Point::zero(), DISPLAY_SIZE)
            .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off))
            .draw(target)?;

        let play_def = RhythmDefinition::try_from(self.states).unwrap();
        // self.analysis.set_rhythm(play_def.clone());
        // self.analysis.step(self.ringing);
        self.music = play_def.to_music(&Vec::new()).unwrap(); // TODO: only recalc on changed toggle switch?

        self.staff.draw(
            target,
            &[
                StaffElement::Clef(Clef::Bass),
                StaffElement::Music(&self.music),
            ],
        )?;

        // self.analysis.draw(target, Point { x: 0, y: 50 })?;
        // self.states.draw(target, Point { x: 0, y: 0 })?;

        self.menu.draw(target, Point { x: 0, y: 64 - 10 })?;

        Ok(())
    }

    /// Read buttons and update states accordingly, returns synth commands that are based
    /// on user input and that have to be handled ASAP.
    pub fn update_io_state(&mut self, new_state: IOState) -> Vec<Command, 4> {
        self.io_state = new_state;

        self.states.set_all(new_state.toggle_switches);

        self.menu.update(new_state);

        let mut music_array = Vec::new();
        music_array.clone_from(&self.music);
        self.menu.synth_controller.set_music(music_array);

        let command = self
            .fretting_and_plucking
            .action(new_state.playing_buttons)
            .and_then(|action| self.action_to_command.translate(action));

        command.into_iter().collect::<Vec<_, 4>>()
    }

    /// Gets the next command for the synth, steps forward in sixteenth notes.
    /// This function is very timing sensitive and should be called in regular intervals,
    /// using timer functionality or the rhythm will not be correct.
    pub fn next_synth_command(&mut self) -> Vec<Command, 4> {
        self.menu.next_command()
    }

    /// So the driving main knows at how many _sixteenths_ per minute to drive the interface.
    pub fn spm(&self) -> u32 {
        let bpm = self.menu.bpm();
        bpm * 4
    }
}
