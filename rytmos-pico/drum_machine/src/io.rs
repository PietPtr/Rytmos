use drum_machine_bsp::{
    hal::{
        adc::AdcPin,
        gpio::{
            self, DynFunction, DynPinId, FunctionSioInput, FunctionSioOutput, Pin, PullDown,
            PullNone,
        },
        Adc,
    },
    Pins,
};
use embedded_hal::digital::v2::{OutputPin, ToggleableOutputPin};
use embedded_hal::{adc::OneShot, digital::v2::InputPin};

use crate::cd4051::Cd4051Addressor;

#[derive(Debug, Default, Clone, Copy, defmt::Format)]
pub struct BoardSettings {
    pub play_or_pause: bool,
    pub countoff_at_play: bool,
    pub leds_enabled: bool,
    pub time_signature: bool,
    pub cymbal_every_four_measures: bool,
    pub reserved0: bool,
    pub reserved1: bool,
    pub reserved2: bool,
}

impl From<[bool; 8]> for BoardSettings {
    fn from(value: [bool; 8]) -> Self {
        Self {
            play_or_pause: value[3],
            countoff_at_play: value[2],
            leds_enabled: value[1],
            time_signature: value[0],
            cymbal_every_four_measures: value[7],
            reserved0: value[6],
            reserved1: value[5],
            reserved2: value[4],
        }
    }
}

#[derive(Debug, Default, Clone, Copy, defmt::Format)]
pub struct DrumIOState {
    pub hat: [bool; 16],
    pub snare: [bool; 16],
    pub kick: [bool; 16],
    pub active_led: Option<u8>,
    pub volume: [u16; 3],
    pub expr: [u16; 3],
    pub filter: [u16; 3],
    pub bpm: u16,
    pub settings: BoardSettings,
}

/// Holds the state of all IO on the drum machine board and contains functions to modify and read it
pub struct DrumIO {
    state: DrumIOState,
    perc_addr: Cd4051Addressor,
    pot_addr: Cd4051Addressor,
    led_addr: Cd4051Addressor,
    led_drivers: [Pin<DynPinId, FunctionSioOutput, PullNone>; 2],
    pot_readers: [AdcPin<Pin<DynPinId, DynFunction, PullDown>>; 2],
    perc_bus_read_pins: [Pin<DynPinId, FunctionSioInput, PullNone>; 7],
    adc: Adc,
    led_pin: Pin<DynPinId, FunctionSioOutput, PullNone>,
}

impl DrumIO {
    pub fn new(pins: Pins, adc: Adc) -> Self {
        let led_pin = pins
            .led
            .into_push_pull_output_in_state(gpio::PinState::High)
            .reconfigure()
            .into_dyn_pin();

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

        let perc_bus_read_pins = [
            pins.hat_read_1_2.reconfigure().into_dyn_pin(),
            pins.hat_read_3_4.reconfigure().into_dyn_pin(),
            pins.snare_read_1_2.reconfigure().into_dyn_pin(),
            pins.snare_read_3_4.reconfigure().into_dyn_pin(),
            pins.kick_read_1_2.reconfigure().into_dyn_pin(),
            pins.kick_read_3_4.reconfigure().into_dyn_pin(),
            pins.settings_read.reconfigure().into_dyn_pin(),
        ];

        Self {
            perc_addr,
            pot_addr,
            led_addr,
            led_drivers,
            pot_readers,
            perc_bus_read_pins,
            adc,
            state: DrumIOState::default(),
            led_pin,
        }
    }

    pub fn led_index(&mut self, index: u8) {
        self.state.active_led = Some(index)
    }

    pub fn disable_led(&mut self) {
        self.state.active_led = None
    }

    /// Commits the current state to the outputs, reads the on-board state of the inputs
    pub fn update(&mut self) -> DrumIOState {
        self.led_pin.toggle().unwrap();

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
        } else {
            self.led_drivers[0].set_low().unwrap();
            self.led_drivers[1].set_low().unwrap();
        }

        // Read the potentiometers
        let mut pot_values = [0; 16];

        for addr in 0..8 {
            self.pot_addr.set(addr);

            let read0: u16 = self.adc.read(&mut self.pot_readers[0]).unwrap();
            let read1: u16 = self.adc.read(&mut self.pot_readers[1]).unwrap();

            pot_values[addr as usize] = read0;
            pot_values[8 + addr as usize] = read1;
        }

        // Read the settings and the percussion configuration
        let mut hat_values = [false; 16];
        let mut snare_values = [false; 16];
        let mut kick_values = [false; 16];
        let mut setting_values = [false; 8];

        for addr in 0..8 {
            self.perc_addr.set(addr);

            let addr_low = addr as usize;
            let addr_high = 8 + addr as usize;

            hat_values[addr_low] = self.perc_bus_read_pins[0].is_high().unwrap();
            hat_values[addr_high] = self.perc_bus_read_pins[1].is_high().unwrap();
            snare_values[addr_low] = self.perc_bus_read_pins[2].is_high().unwrap();
            snare_values[addr_high] = self.perc_bus_read_pins[3].is_high().unwrap();
            kick_values[addr_low] = self.perc_bus_read_pins[4].is_high().unwrap();
            kick_values[addr_high] = self.perc_bus_read_pins[5].is_high().unwrap();
            setting_values[addr_low] = self.perc_bus_read_pins[6].is_high().unwrap();
        }

        // Update the io state struct with the raw values
        self.state.volume.copy_from_slice(&pot_values[0..3]);
        self.state.expr.copy_from_slice(&pot_values[3..6]);
        self.state.filter.copy_from_slice(&pot_values[6..9]);
        self.state.bpm = 4095 - pot_values[9];
        self.state.hat = hat_values;
        self.state.snare = snare_values;
        self.state.kick = kick_values;
        self.state.settings = setting_values.into();

        self.state
    }
}
