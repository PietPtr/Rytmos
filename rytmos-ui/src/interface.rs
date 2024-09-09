use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};
use heapless::Vec;
use rytmos_engrave::staff::{Clef, Music, Staff, StaffElement};
use rytmos_scribe::sixteen_switches::{MeasureState, RhythmDefinition, SwitchState};
use rytmos_synth::commands::Command;

use crate::{
    play_analysis::PlayAnalysis,
    synth_controller::{SynthController, SynthControllerSettings},
};

pub const DISPLAY_SIZE: Size = Size::new(128, 64);

#[derive(Default, Debug, Copy, Clone)]
pub struct IOState {
    /// The sixteen tri-state toggle switches for defining rhythms
    pub toggle_switches: [SwitchState; 16],
    /// Four buttons for the fretting hand
    pub fretting_buttons: [bool; 4],
    /// Two buttons for string plucking
    pub plucking_buttons: [bool; 2],
    /// Four buttons below the screen for menu navigation
    pub menu_buttons: [bool; 4],
}

/// The top level of all interfaces in the device.
pub struct Interface {
    // Gadgets, drawables
    staff: Staff,
    analysis: PlayAnalysis,
    states: MeasureState,
    synth_controller: SynthController,

    // IO related
    io_state: IOState, // TODO: really necessary to store?

    // Logic state, maybe should be empty as this is state for inside gadgets?
    ringing: bool,
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
            analysis: PlayAnalysis::new(RhythmDefinition::default()),
            states: MeasureState::default(),
            synth_controller: SynthController::new(SynthControllerSettings::default()),
            io_state: IOState::default(),
            ringing: false,
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
        self.analysis.set_rhythm(play_def.clone());
        self.analysis.step(self.ringing);
        self.music = play_def.to_music(&Vec::new()).unwrap(); // TODO: only recalc on changed toggle switch?

        self.staff.draw(
            target,
            &[
                StaffElement::Clef(Clef::Bass),
                StaffElement::Music(&self.music),
            ],
        )?;

        self.analysis.draw(target, Point { x: 0, y: 50 })?;
        self.states.draw(target, Point { x: 0, y: 0 })?;

        Ok(())
    }

    pub fn update_io_state(&mut self, new_state: IOState) {
        self.io_state = new_state;

        self.states.set_all(new_state.toggle_switches);

        // TODO: feed fretting and plucking switches into play analysis and update that with the playing logic.
        // TODO: define the menus
    }

    /// Gets the next command for the synth, given the time t in 128th notes.
    pub fn next_synth_command(&mut self, t: u64) -> Vec<Command, 4> {
        self.synth_controller.command_for_time(t)
    }

    // TODO: will be set by a menu, should be retrieved by main to call update correctly
    pub fn bpm(&self) -> u32 {
        80
    }
}
