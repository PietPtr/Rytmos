use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};
use rytmos_engrave::staff::{Clef, Staff, StaffElement};
use rytmos_scribe::sixteen_switches::{MeasureState, PlayDefinition, SwitchState};

use crate::play_analysis::PlayAnalysis;

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

    // IO related
    io_state: IOState, // TODO: really necessary to store?

    // Logic state, maybe should be empty as this is state for inside gadgets?
    ringing: bool,
}

impl Interface {
    pub fn new() -> Self {
        Self {
            staff: Staff::new(DISPLAY_SIZE.width, Point::new(0, 0)),
            analysis: PlayAnalysis::new(PlayDefinition::default()),
            states: MeasureState::default(),
            ringing: false,
            io_state: IOState::default(),
        }
    }

    pub fn draw<D>(&mut self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = BinaryColor>,
    {
        Rectangle::new(Point::zero(), DISPLAY_SIZE)
            .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off))
            .draw(target)?;

        let play_def = PlayDefinition::try_from(self.states).unwrap();
        self.analysis.set_rhythm(play_def.clone());
        self.analysis.step(self.ringing);
        let music = play_def.to_music().unwrap();

        self.staff.draw(
            target,
            &[StaffElement::Clef(Clef::Bass), StaffElement::Music(&music)],
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
}
