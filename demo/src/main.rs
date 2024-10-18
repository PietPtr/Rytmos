use std::time::Instant;

use defmt::info;
use defmt_rtt as _;
use embedded_graphics::{pixelcolor::BinaryColor, prelude::*};
use embedded_graphics_simulator::{
    sdl2::{Keycode, Mod},
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use env_logger::{Builder, Env};
use log::LevelFilter;
use rytmos_scribe::sixteen_switches::SwitchState;
use rytmos_ui::{
    interface::{IOState, Interface, PlayingButtons},
    play_analysis::PlayAnalysis,
};

fn main() -> Result<(), core::convert::Infallible> {
    Builder::from_env(Env::default().default_filter_or(LevelFilter::Trace.to_string())).init();

    let display_size = Size::new(128, 64);
    let mut display = SimulatorDisplay::<BinaryColor>::new(display_size);

    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::OledWhite)
        .scale(8)
        .pixel_spacing(1)
        .build();
    let mut window = Window::new("Rytmos", &output_settings);

    let mut states = [SwitchState::Noop; 16];
    let mut playing_buttons = PlayingButtons {
        fretting_buttons: [false; 4],
        plucking_buttons: [false; 2],
    };
    let mut menu_buttons = [false; 4];

    let mut interface = Interface::new();

    let mut now = Instant::now();

    'main: loop {
        let io_state = IOState {
            toggle_switches: states,
            playing_buttons,
            menu_buttons,
        };

        interface.draw(&mut display)?;
        let synth_commands = interface.update_io_state(io_state);
        if !synth_commands.is_empty() {
            log::info!("synth commands for input: {:?}", synth_commands);
        }

        window.update(&display);

        for event in window.events() {
            match event {
                SimulatorEvent::KeyDown {
                    keycode,
                    keymod,
                    repeat: false,
                } => {
                    update_toggle_switches_states(&mut states, keycode, keymod);
                    update_playing_buttons(&mut playing_buttons, keycode, true);
                    update_menu_buttons(&mut menu_buttons, keycode, true);
                }
                SimulatorEvent::KeyUp {
                    keycode,
                    keymod: _,
                    repeat: false,
                } => {
                    update_playing_buttons(&mut playing_buttons, keycode, false);
                    update_menu_buttons(&mut menu_buttons, keycode, false);
                }
                SimulatorEvent::Quit => break 'main,
                _ => (),
            }
        }

        while now.elapsed().as_millis()
            < PlayAnalysis::step_size_ms(interface.spm().max(10)) as u128
        {}
        now = Instant::now();

        let next_player_commands = interface.next_synth_command();
    }

    Ok(())
}

fn update_menu_buttons(menu_buttons: &mut [bool; 4], keycode: Keycode, down: bool) {
    match keycode {
        Keycode::Num9 => menu_buttons[0] = down,
        Keycode::Num0 => menu_buttons[1] = down,
        Keycode::LeftBracket => menu_buttons[2] = down,
        Keycode::RightBracket => menu_buttons[3] = down,
        _ => (),
    }
}

fn update_playing_buttons(playing_buttons: &mut PlayingButtons, keycode: Keycode, down: bool) {
    match keycode {
        Keycode::Semicolon => playing_buttons.fretting_buttons[0] = down,
        Keycode::Q => playing_buttons.fretting_buttons[1] = down,
        Keycode::J => playing_buttons.fretting_buttons[2] = down,
        Keycode::K => playing_buttons.fretting_buttons[3] = down,
        Keycode::W => playing_buttons.plucking_buttons[0] = down,
        Keycode::V => playing_buttons.plucking_buttons[1] = down,
        _ => (),
    }
}

fn find_keycode_position(
    keymod: embedded_graphics_simulator::sdl2::Mod,
    keycode: Keycode,
) -> Option<(usize, SwitchState)> {
    const SWITCHES: [[Keycode; 3]; 8] = [
        [Keycode::Num1, Keycode::Quote, Keycode::A],
        [Keycode::Num2, Keycode::Comma, Keycode::O],
        [Keycode::Num3, Keycode::Period, Keycode::E],
        [Keycode::Num4, Keycode::P, Keycode::U],
        [Keycode::Num5, Keycode::Y, Keycode::I],
        [Keycode::Num6, Keycode::F, Keycode::D],
        [Keycode::Num7, Keycode::G, Keycode::H],
        [Keycode::Num8, Keycode::C, Keycode::T],
    ];

    for (row_index, row) in SWITCHES.iter().enumerate() {
        if let Some(col_index) = row.iter().position(|&k| k == keycode) {
            let play = match col_index {
                0 => SwitchState::Atck,
                1 => SwitchState::Noop,
                2 => SwitchState::Mute,
                _ => return None,
            };
            let sixteenth = if keymod.contains(Mod::LSHIFTMOD) {
                row_index + 8
            } else {
                row_index
            };
            return Some((sixteenth, play));
        }
    }
    None
}

fn update_toggle_switches_states(
    states: &mut [SwitchState; 16],
    keycode: Keycode,
    keymod: embedded_graphics_simulator::sdl2::Mod,
) {
    if let Some((sixteenth, state)) = find_keycode_position(keymod, keycode) {
        states[sixteenth] = state;
    }
}
