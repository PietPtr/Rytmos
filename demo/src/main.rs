use std::time::Instant;

use defmt_rtt as _;
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};
use embedded_graphics_simulator::{
    sdl2::{Keycode, Mod},
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use env_logger::{Builder, Env};
use log::LevelFilter;
use rytmos_engrave::staff::{Clef, Staff, StaffElement};
use rytmos_scribe::sixteen_switches::{MeasureState, PlayDefinition, SwitchState};
use rytmos_ui::{
    interface::{IOState, Interface},
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
    let mut ringing = false;

    let mut interface = Interface::new();

    let mut now = Instant::now();

    'main: loop {
        let io_state = IOState {
            toggle_switches: states,
            fretting_buttons: [false; 4],
            plucking_buttons: [false; 2],
            menu_buttons: [false; 4],
        };

        interface.draw(&mut display)?;
        interface.update_io_state(io_state);

        window.update(&display);

        for event in window.events() {
            match event {
                SimulatorEvent::KeyDown {
                    keycode,
                    keymod,
                    repeat: false,
                } => {
                    mod_state(&mut states, keycode, keymod);
                }
                SimulatorEvent::KeyUp {
                    keycode,
                    keymod: _,
                    repeat: false,
                } => {}
                SimulatorEvent::Quit => break 'main,
                _ => (),
            }
        }

        while now.elapsed().as_millis() < PlayAnalysis::step_size_ms(60) as u128 {}
        now = Instant::now();
    }

    Ok(())
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

fn mod_state(
    states: &mut [SwitchState; 16],
    keycode: Keycode,
    keymod: embedded_graphics_simulator::sdl2::Mod,
) {
    if let Some((sixteenth, state)) = find_keycode_position(keymod, keycode) {
        states[sixteenth] = state;
    }
}
