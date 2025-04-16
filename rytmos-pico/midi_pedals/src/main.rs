#![no_std]
#![no_main]

#[link_section = ".boot2"]
#[no_mangle]
#[used]
pub static BOOT2_FIRMWARE: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

use common::debouncer::Debouncer;
use cortex_m::asm;
use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::v2::InputPin;
use embedded_hal::digital::v2::OutputPin;
use panic_probe as _;
use rp_pico::{
    entry,
    hal::{
        clocks::{init_clocks_and_plls, Clock, ClocksManager},
        gpio::{self, FunctionSio, Pin, PullUp, SioInput},
        sio::Sio,
        usb::UsbBus,
        Watchdog,
    },
    pac,
};
use usb_device::device::StringDescriptors;
use usb_device::{
    bus::UsbBusAllocator,
    device::{UsbDeviceBuilder, UsbVidPid},
};
use usbd_midi::message::U7;
use usbd_midi::CableNumber;
use usbd_midi::UsbMidiEventPacket;
use usbd_midi::{
    message::{Channel, Message, Note},
    UsbMidiClass, UsbMidiPacketReader,
};

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let sio = Sio::new(pac.SIO);

    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let clocks = init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    info!("Creating usb devices.");

    let usb_bus = UsbBusAllocator::new(UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));

    let mut led_pin = pins.gpio25.into_push_pull_output();
    let pedal_pin: Pin<_, FunctionSio<SioInput>, PullUp> = pins.gpio16.reconfigure();

    let mut midi = UsbMidiClass::new(&usb_bus, 1, 0).unwrap();

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0xabc9, 0xee0))
        .device_class(0)
        .device_sub_class(0)
        .strings(&[StringDescriptors::default()
            .manufacturer("de vck")
            .product("midi pedals lmao")
            .serial_number("0")])
        .unwrap()
        .build();

    info!("usb device is created");

    let mut pedal_debouncer = Debouncer::new(5000);

    led_pin.set_high().unwrap();

    loop {
        pedal_debouncer.update(pedal_pin.is_high().unwrap());

        usb_dev.poll(&mut [&mut midi]);

        if pedal_debouncer.stable_falling_edge() {
            midi.send_packet(
                Message::NoteOn(Channel::Channel1, Note::C4, U7::MAX)
                    .into_packet(CableNumber::Cable0),
            )
            .ok();
        }

        if pedal_debouncer.stable_rising_edge() {
            midi.send_packet(
                Message::NoteOff(Channel::Channel1, Note::C4, U7::MIN)
                    .into_packet(CableNumber::Cable0),
            )
            .ok();
        }
    }
}
