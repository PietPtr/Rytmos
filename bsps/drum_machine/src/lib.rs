#![no_std]

pub use rp2040_hal as hal;

#[cfg(feature = "rt")]
extern crate cortex_m_rt;

#[cfg(feature = "rt")]
pub use hal::entry;

#[cfg(feature = "boot2")]
#[link_section = ".boot2"]
#[no_mangle]
#[used]
pub static BOOT2_FIRMWARE: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

pub use hal::pac;

hal::bsp_pins!(
    Gpio0 { name: pot_addr0 },
    Gpio1 { name: pot_addr1 },
    Gpio2 { name: pot_addr2 },
    Gpio3 { name: led_addr0 },
    Gpio4 { name: led_addr1 },
    Gpio5 { name: led_addr2 },
    Gpio6 {
        name: gpio6_disconnected
    },
    Gpio7 { name: led_drive1 },
    Gpio8 { name: led_drive0 },
    Gpio9 {
        name: settings_read
    },
    Gpio10 { name: hat_read_3_4 },
    Gpio11 { name: hat_read_1_2 },
    Gpio12 { name: i2s_sck },
    Gpio13 { name: i2s_din },
    Gpio14 { name: i2s_bck },
    Gpio15 { name: i2s_lrck },
    Gpio16 {
        name: snare_read_3_4
    },
    Gpio17 {
        name: snare_read_1_2
    },
    Gpio18 {
        name: kick_read_3_4
    },
    Gpio19 {
        name: kick_read_1_2
    },
    Gpio20 { name: perc_addr2 },
    Gpio21 { name: perc_addr1 },
    Gpio22 { name: perc_addr0 },
    Gpio23 { name: gpio23 },
    Gpio24 { name: gpio24 },
    Gpio25 { name: led },
    Gpio26 { name: pot_read1 },
    Gpio27 { name: pot_read0 },
    Gpio28 { name: gpio28 },
    Gpio29 { name: gpio29 },
);
