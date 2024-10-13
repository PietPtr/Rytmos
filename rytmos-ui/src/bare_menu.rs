use crate::{interface::IOState, synth_controller::SynthController};

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
///         FRET3: inc metronome volume
///         FRET4: dec metronome volume
///         PLUCK_LEFT: -
///         PLUCK_RIGHT: -
pub struct BareMenu<'a> {
    tempo: u8,
    synth_controller: &'a mut SynthController,
    last_state: IOState,
    play_mode: PlayMode,
}

impl<'a> BareMenu<'a> {
    pub fn new(synth_controller: &'a mut SynthController) -> Self {
        Self {
            tempo: 80,
            synth_controller,
            last_state: IOState::default(), // TODO: also do this in playing.rs?
            play_mode: PlayMode::default(),
        }
    }

    // pub fn draw() // TODO: this

    pub fn update(&mut self, state: IOState) {
        macro_rules! is_button_pressed {
            ($i:expr) => {
                matches!(
                    (self.last_state.menu_buttons[$i], state.menu_buttons[$i]),
                    (true, false)
                )
            };
        }

        let button1 = is_button_pressed!(0);
        let button2 = is_button_pressed!(1);
        let button3 = is_button_pressed!(2);
        let button4 = is_button_pressed!(3);

        if button1 {
            self.synth_controller.play_or_stop_toggle();
        }

        if button2 {
            self.play_mode = self.play_mode.next()
        }

        if button3 {
            self.synth_controller.
        }
    }
}
