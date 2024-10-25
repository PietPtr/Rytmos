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
    pub synth_controller: SynthController,
    last_state: IOState,
    play_mode: PlayMode,
    saved_metronome_tempo: u8,
    metronome_enabled: bool,
}

impl BareMenu {
    pub fn new() -> Self {
        let mut s = Self {
            synth_controller: SynthController::new(SynthControllerSettings::default()),
            last_state: IOState::default(), // TODO: also do this in playing.rs?
            play_mode: PlayMode::default(),
            saved_metronome_tempo: 60,
            metronome_enabled: false,
        };

        s.apply_play_mode();

        s
    }

    pub fn draw<D>(&self, target: &mut D, position: Point) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = BinaryColor>,
    {
        let play_pause_position = position + Point { x: 10, y: 0 };

        if self.synth_controller.playing() {
            rytmos_symbols::draw_symbol(target, play_pause_position, rytmos_symbols::PLAYING)?;
        } else {
            rytmos_symbols::draw_symbol(target, play_pause_position, rytmos_symbols::PAUSED)?;
        }

        let playmode_position = position + Point { x: 46, y: 0 };

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

        let metronome_position = position + Point { x: 79, y: 0 };

        if self.metronome_enabled {
            let beat = (self.synth_controller.beat() * 4.) as u64;

            let symbol = match beat {
                0 => rytmos_symbols::METRONOME_CENTER,
                1 => rytmos_symbols::METRONOME_LEFT,
                2 => rytmos_symbols::METRONOME_LEFT,
                3 => rytmos_symbols::METRONOME_CENTER,
                4 => rytmos_symbols::METRONOME_CENTER,
                5 => rytmos_symbols::METRONOME_RIGHT,
                6 => rytmos_symbols::METRONOME_RIGHT,
                7 => rytmos_symbols::METRONOME_CENTER,
                8 => rytmos_symbols::METRONOME_CENTER,
                9 => rytmos_symbols::METRONOME_LEFT,
                10 => rytmos_symbols::METRONOME_LEFT,
                11 => rytmos_symbols::METRONOME_CENTER,
                12 => rytmos_symbols::METRONOME_CENTER,
                13 => rytmos_symbols::METRONOME_RIGHT,
                14 => rytmos_symbols::METRONOME_RIGHT,
                15 => rytmos_symbols::METRONOME_CENTER,
                _ => panic!("unexpected symbol beat {beat}"),
            };

            rytmos_symbols::draw_symbol(target, metronome_position, symbol)?;
        } else {
            rytmos_symbols::draw_symbol(
                target,
                metronome_position,
                rytmos_symbols::METRONOME_CENTER,
            )?;
        }

        Ok(())
    }

    fn apply_play_mode(&mut self) {
        match self.play_mode {
            PlayMode::PlayPattern => {
                self.synth_controller
                    .update_settings(SynthControllerSettingsUpdate {
                        play_pattern: Some(true),
                        measures_silence: Some(0),
                        ..Default::default()
                    });
            }
            PlayMode::PatternEveryOtherBar => {
                self.synth_controller
                    .update_settings(SynthControllerSettingsUpdate {
                        play_pattern: Some(true),
                        measures_silence: Some(1),
                        ..Default::default()
                    });
            }
            PlayMode::NeverPlayPattern => {
                self.synth_controller
                    .update_settings(SynthControllerSettingsUpdate {
                        play_pattern: Some(false),
                        ..Default::default()
                    });
            }
        }
    }

    pub fn update(&mut self, state: IOState) {
        macro_rules! was_menu_button_pressed {
            ($i:expr) => {{
                matches!(
                    (self.last_state.menu_buttons[$i], state.menu_buttons[$i]),
                    (true, false)
                )
            }};
        }

        macro_rules! _was_fret_pressed {
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

        if button1 {
            self.synth_controller.play_or_stop_toggle();
        }

        if button2 {
            self.play_mode = self.play_mode.next();
            self.apply_play_mode();
        }

        if button3 {
            self.metronome_enabled = !self.metronome_enabled;

            self.synth_controller
                .update_settings(SynthControllerSettingsUpdate {
                    metronome: Some(self.metronome_enabled),
                    ..Default::default()
                });
        }

        if state.menu_buttons[3] {
            if state.playing_buttons.fretting_buttons[0] {
                self.saved_metronome_tempo = self.saved_metronome_tempo.saturating_add(1);
            }

            if state.playing_buttons.fretting_buttons[1] {
                self.saved_metronome_tempo = self.saved_metronome_tempo.saturating_sub(1).max(10);
            }
        }

        self.last_state = state;
    }

    pub(crate) fn next_command(&mut self) -> Vec<Command, 4> {
        self.synth_controller.next_command()
    }

    pub(crate) fn bpm(&self) -> u32 {
        self.saved_metronome_tempo as u32
    }
}

impl Default for BareMenu {
    fn default() -> Self {
        Self::new()
    }
}
