use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::{DrawTarget, Point},
};
use heapless::Vec;
use rytmos_synth::commands::Command;

use crate::{
    interface::IOState,
    synth_controller::{SynthController, SynthControllerSettings, SynthControllerSettingsUpdate},
};

#[derive(Debug, Default)]
#[repr(u8)]
enum PlayMode {
    #[default]
    PlayPattern = 0,
    PatternEveryOtherBar = 1,
    NeverPlayPattern = 2,
}

impl PlayMode {
    fn next(&self) -> Self {
        match self {
            PlayMode::PlayPattern => PlayMode::PatternEveryOtherBar,
            PlayMode::PatternEveryOtherBar => PlayMode::NeverPlayPattern,
            PlayMode::NeverPlayPattern => PlayMode::PlayPattern,
        }
    }
}

/// Implementation of a very simple menu:
/// - menu button functions:
///     - play / stopped, shows play or stopped icon
///     - cycle through modes, shows M(mode number):
///         1) always play pattern
///         2) play pattern every other bar
///         3) never play pattern
///     - metronome enable/disable, shows (moving) metronome icon
///     - fn
///         FRET1: inc bpm
///         FRET2: dec bpm
///         FRET3: TODO: inc metronome volume
///         FRET4: TODO: dec metronome volume
///         PLUCK_LEFT: -
///         PLUCK_RIGHT: -
pub struct BareMenu {
    synth_controller: SynthController,
    last_state: IOState,
    play_mode: PlayMode,
    saved_metronome_tempo: u8,
    metronome_enabled: bool,
}

impl BareMenu {
    pub fn new() -> Self {
        Self {
            synth_controller: SynthController::new(SynthControllerSettings::default()),
            last_state: IOState::default(), // TODO: also do this in playing.rs?
            play_mode: PlayMode::default(),
            saved_metronome_tempo: 80,
            metronome_enabled: false,
        }
    }

    pub fn draw<D>(&self, target: &mut D, position: Point) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = BinaryColor>,
    {
        let playmode_position = position + Point { x: 50, y: 0 };

        match self.play_mode {
            PlayMode::PlayPattern => {
                rytmos_symbols::draw_symbol(target, playmode_position, rytmos_symbols::LETTER_A)?;
            }
            PlayMode::PatternEveryOtherBar => {
                rytmos_symbols::draw_symbol(target, playmode_position, rytmos_symbols::LETTER_B)?;
            }
            PlayMode::NeverPlayPattern => {
                rytmos_symbols::draw_symbol(target, playmode_position, rytmos_symbols::LETTER_C)?;
            }
        }

        Ok(())
    }

    pub fn update(&mut self, state: IOState) {
        macro_rules! was_menu_button_pressed {
            ($i:expr) => {{
                log::info!(
                    "{} {} {}",
                    $i,
                    self.last_state.menu_buttons[$i],
                    state.menu_buttons[$i]
                );
                matches!(
                    (self.last_state.menu_buttons[$i], state.menu_buttons[$i]),
                    (true, false)
                )
            }};
        }

        macro_rules! was_fret_pressed {
            ($i:expr) => {
                matches!(
                    (
                        self.last_state.playing_buttons.fretting_buttons[$i],
                        state.playing_buttons.fretting_buttons[$i]
                    ),
                    (true, false)
                )
            };
        }

        let button1 = was_menu_button_pressed!(0);
        let button2 = was_menu_button_pressed!(1);
        let button3 = was_menu_button_pressed!(2);
        let button4 = was_menu_button_pressed!(3);

        log::info!("{} {} {} {}", button1, button2, button3, button4);

        let mut metronome_changed = false;

        if button1 {
            self.synth_controller.play_or_stop_toggle();
        }

        if button2 {
            self.play_mode = self.play_mode.next()
        }

        if button3 {
            self.metronome_enabled = !self.metronome_enabled;

            metronome_changed = true;
        }

        if button4 {
            let fret1 = was_fret_pressed!(0);
            let fret2 = was_fret_pressed!(1);

            if fret1 {
                metronome_changed = true;
                (self.saved_metronome_tempo, _) = self.saved_metronome_tempo.overflowing_add(1);
            }

            if fret2 {
                metronome_changed = true;
                (self.saved_metronome_tempo, _) = self.saved_metronome_tempo.overflowing_sub(1);
            }
        }

        if metronome_changed {
            let metronome_setting = if self.metronome_enabled {
                Some(self.saved_metronome_tempo)
            } else {
                None
            };

            self.synth_controller
                .update_settings(SynthControllerSettingsUpdate {
                    metronome: Some(metronome_setting),
                    ..Default::default()
                });
        }

        self.last_state = state;
    }

    pub(crate) fn next_command(&mut self) -> Vec<Command, 4> {
        self.synth_controller.next_command()
    }
}

impl Default for BareMenu {
    fn default() -> Self {
        Self::new()
    }
}
