use crate::{
    interface::IOState,
    synth_controller::{SynthController, SynthControllerSettingsUpdate},
};

#[derive(Debug, Default)]
#[repr(u8)]
enum PlayMode {
    PlayPattern = 0,
    #[default]
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
pub struct BareMenu<'a> {
    synth_controller: &'a mut SynthController,
    last_state: IOState,
    play_mode: PlayMode,
    saved_metronome_tempo: u8,
    metronome_enabled: bool,
}

impl<'a> BareMenu<'a> {
    pub fn new(synth_controller: &'a mut SynthController) -> Self {
        Self {
            synth_controller,
            last_state: IOState::default(), // TODO: also do this in playing.rs?
            play_mode: PlayMode::default(),
            saved_metronome_tempo: 80,
            metronome_enabled: false,
        }
    }

    // pub fn draw() // TODO: this

    pub fn update(&mut self, state: IOState) {
        macro_rules! was_menu_button_pressed {
            ($i:expr) => {
                matches!(
                    (self.last_state.menu_buttons[$i], state.menu_buttons[$i]),
                    (true, false)
                )
            };
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
    }
}
