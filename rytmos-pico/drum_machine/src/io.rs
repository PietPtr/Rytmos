use drum_machine_bsp::{
    hal::{
        adc::AdcPin,
        gpio::{DynPinId, FunctionNull, FunctionSioOutput, Pin, PullDown, PullNone},
        Adc,
    },
    Pins,
};
use embedded_hal::adc::OneShot;
use embedded_hal::digital::v2::OutputPin;

use crate::cd4051::Cd4051Addressor;

#[derive(Debug, Default, Clone, Copy)]
pub struct BoardSettings {
    play_or_pause: bool,
    countoff_at_play: bool,
    leds_enabled: bool,
    time_signature: bool,
    cymbal_every_four_measures: bool,
    reserved0: bool,
    reserved1: bool,
    reserved2: bool,
}

impl From<[bool; 8]> for BoardSettings {
    fn from(value: [bool; 8]) -> Self {
        Self {
            play_or_pause: value[0],
            countoff_at_play: value[1],
            leds_enabled: value[2],
            time_signature: value[3],
            cymbal_every_four_measures: value[4],
            reserved0: value[5],
            reserved1: value[6],
            reserved2: value[7],
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct DrumIOState {
    hat: [bool; 16],
    snare: [bool; 16],
    kick: [bool; 16],
    active_led: Option<u8>,
    volume: [i16; 3],
    filter: [i16; 3],
    expr: [i16; 3],
    bpm: i16,
    settings: BoardSettings,
}

/// Holds the state of all IO on the drum machine board and contains functions to modify and read it
pub struct DrumIO {
    pub state: DrumIOState,
    perc_addr: Cd4051Addressor,
    pot_addr: Cd4051Addressor,
    led_addr: Cd4051Addressor,
    led_drivers: [Pin<DynPinId, FunctionSioOutput, PullNone>; 2],
    pot_readers: [AdcPin<Pin<DynPinId, FunctionNull, PullDown>>; 2],
    adc: Adc,
}

impl DrumIO {
    pub fn new(pins: Pins, adc: Adc) -> Self {
        let perc_addr = Cd4051Addressor {
            addr0: pins.perc_addr0.reconfigure().into_dyn_pin(),
            addr1: pins.perc_addr1.reconfigure().into_dyn_pin(),
            addr2: pins.perc_addr2.reconfigure().into_dyn_pin(),
        };

        let pot_addr = Cd4051Addressor {
            addr0: pins.pot_addr0.reconfigure().into_dyn_pin(),
            addr1: pins.pot_addr1.reconfigure().into_dyn_pin(),
            addr2: pins.pot_addr2.reconfigure().into_dyn_pin(),
        };

        let led_addr = Cd4051Addressor {
            addr0: pins.led_addr0.reconfigure().into_dyn_pin(),
            addr1: pins.led_addr1.reconfigure().into_dyn_pin(),
            addr2: pins.led_addr2.reconfigure().into_dyn_pin(),
        };

        let led_drivers = [
            pins.led_drive0.reconfigure().into_dyn_pin(),
            pins.led_drive1.reconfigure().into_dyn_pin(),
        ];

        let pot_readers = [
            AdcPin::new(pins.pot_read0.reconfigure().into_dyn_pin()).unwrap(),
            AdcPin::new(pins.pot_read1.reconfigure().into_dyn_pin()).unwrap(),
        ];

        Self {
            perc_addr,
            pot_addr,
            led_addr,
            led_drivers,
            pot_readers,
            adc,
            state: DrumIOState::default(),
        }
    }

    /// Commits the current state to the outputs, reads the on-board state of the inputs
    pub fn update(&mut self) {
        // Update which LED is on
        if let Some(led_id) = self.state.active_led {
            let led_id = led_id & 0b1111;
            if led_id >> 3 == 1 {
                self.led_drivers[0].set_low().unwrap();
                self.led_drivers[1].set_high().unwrap();
            } else {
                self.led_drivers[0].set_high().unwrap();
                self.led_drivers[1].set_low().unwrap();
            }

            self.led_addr.set(led_id & 0b111);
        }

        // Read the potentiometers
        for addr in 0..8 {
            self.pot_addr.set(addr);
            self.adc.read(&mut self.pot_readers[0]);
            self.adc.read(&mut self.pot_readers[1]);
        }

        // Read the settings and the percussion configuration
    }
}
