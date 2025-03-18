#![no_std]
#![no_main]

#[link_section = ".boot2"]
#[no_mangle]
#[used]
pub static BOOT2_FIRMWARE: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::v2::OutputPin;
use panic_probe as _;
use rp_pico::{
    entry,
    hal::{
        clocks::{Clock, ClocksManager},
        gpio::{self},
        sio::Sio,
    },
    pac,
};

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let sio = Sio::new(pac.SIO);

    let clocks = ClocksManager::new(pac.CLOCKS);

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut led_pin = pins.gpio25.into_push_pull_output();

    info!("Start.");

    // pac.PPB.syst_csr.write(|w| w.clksource().set_bit());
    // pac.PPB.syst_csr.write(|w| w.enable().set_bit());
    pac.PPB.syst_csr.write(|w| unsafe { w.bits(0x5) });
    pac.PPB.syst_rvr.write(|w| unsafe { w.bits(0xffffff) });

    info!(
        "\nclksource={} ({})\nenabled={}\ntickint={}\nrvr={:#x}",
        if pac.PPB.syst_csr.read().clksource().bit() {
            "processor"
        } else {
            "refclock"
        },
        pac.PPB.syst_csr.read().clksource().bit(),
        pac.PPB.syst_csr.read().enable().bit_is_set(),
        pac.PPB.syst_csr.read().tickint().bit(),
        pac.PPB.syst_rvr.read().bits(),
    );

    let points = [
        pac.PPB.syst_cvr.read().current().bits(),
        pac.PPB.syst_cvr.read().current().bits(),
        pac.PPB.syst_cvr.read().current().bits(),
        pac.PPB.syst_cvr.read().current().bits(),
        pac.PPB.syst_cvr.read().current().bits(),
        pac.PPB.syst_cvr.read().current().bits(),
        pac.PPB.syst_cvr.read().current().bits(),
        pac.PPB.syst_cvr.read().current().bits(),
    ];

    pac.PPB
        .syst_cvr
        .write(|w| unsafe { w.current().bits(0xffffff) });

    let after_reset = pac.PPB.syst_cvr.read().current().bits();

    info!("{:x}\n{:x}", points, after_reset);

    loop {
        led_pin.set_high().unwrap();
        delay.delay_ms(1000);
        led_pin.set_low().unwrap();
        delay.delay_ms(1000);
    }
}
