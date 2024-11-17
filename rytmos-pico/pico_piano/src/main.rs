#![no_std]
#![no_main]

#[link_section = ".boot2"]
#[no_mangle]
#[used]
pub static BOOT2_FIRMWARE: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

use core::cell::RefCell;
use core::u32;

use crate::pac::interrupt;
use common::plls;
use cortex_m::{interrupt::Mutex, singleton};
use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::v2::InputPin;
use embedded_hal::digital::v2::OutputPin;
use fixed::types::U8F8;
use fugit::Duration;
use fugit::HertzU32;
use fugit::RateExtU32;
use heapless::Vec;
use micromath::F32Ext;
use panic_probe as _;
use pio_proc::pio_file;
use rp_pico::{
    entry,
    hal::{
        clocks::{Clock, ClockSource, ClocksManager, InitError},
        dma::{double_buffer, DMAExt},
        gpio::{self, FunctionPio0},
        multicore::{Multicore, Stack},
        pio::{Buffers, PIOBuilder, PIOExt, PinDir, ShiftDirection},
        pll::{common_configs::PLL_USB_48MHZ, setup_pll_blocking},
        sio::{Sio, SioFifo},
        timer::{Alarm, Alarm1},
        watchdog::Watchdog,
        xosc::setup_xosc_blocking,
        Timer,
    },
    pac,
};

use rytmos_engrave::ais;
use rytmos_engrave::b;
use rytmos_engrave::cis;
use rytmos_engrave::dis;
use rytmos_engrave::fis;
use rytmos_engrave::gis;
use rytmos_engrave::{a, c, d, e, f, g};
use rytmos_synth::synth::sawtooth::SawtoothSynth;
use rytmos_synth::synth::sawtooth::SawtoothSynthSettings;
use rytmos_synth::{commands::Command, synth::Synth};

const EXTERNAL_XTAL_FREQ_HZ: HertzU32 = HertzU32::from_raw(12_000_000u32);
const RP2040_CLOCK_HZ: HertzU32 = HertzU32::from_raw(307_200_000u32);

const SAMPLE_RATE: HertzU32 = HertzU32::from_raw(24_000u32);
const PIO_INSTRUCTIONS_PER_SAMPLE: u32 = 2;
const NUM_CHANNELS: u32 = 2;
const SAMPLE_RESOLUTION: u32 = 16; // in bits per sample

const I2S_PIO_CLOCK_HZ: HertzU32 = HertzU32::from_raw(
    SAMPLE_RATE.raw() * NUM_CHANNELS * SAMPLE_RESOLUTION * PIO_INSTRUCTIONS_PER_SAMPLE,
);
const I2S_PIO_CLOCKDIV_INT: u16 = (RP2040_CLOCK_HZ.raw() / I2S_PIO_CLOCK_HZ.raw()) as u16;
const I2S_PIO_CLOCKDIV_FRAC: u8 =
    (((RP2040_CLOCK_HZ.raw() % I2S_PIO_CLOCK_HZ.raw()) * 256) / I2S_PIO_CLOCK_HZ.raw()) as u8;

const MCLK_HZ: HertzU32 = HertzU32::from_raw(8 * I2S_PIO_CLOCK_HZ.raw());
const MCLK_CLOCKDIV_INT: u16 = (RP2040_CLOCK_HZ.raw() / MCLK_HZ.raw()) as u16;
const MCLK_CLOCKDIV_FRAC: u8 =
    (((RP2040_CLOCK_HZ.raw() % MCLK_HZ.raw()) * 256) / MCLK_HZ.raw()) as u8;

const BUFFER_SIZE: usize = 16;

static mut CORE1_STACK: Stack<4096> = Stack::new();

fn synth_core(sys_freq: u32) -> ! {
    let mut pac = unsafe { pac::Peripherals::steal() };
    let core = unsafe { pac::CorePeripherals::steal() };
    let mut sio = Sio::new(pac.SIO);
    let pins = gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
    let mut delay = cortex_m::delay::Delay::new(core.SYST, sys_freq);

    let i2s_sck_pin = pins.gpio8.into_function::<FunctionPio0>();
    let i2s_din_pin = pins.gpio9.into_function::<FunctionPio0>();
    let i2s_bck_pin = pins.gpio10.into_function::<FunctionPio0>();
    let i2s_lck_pin = pins.gpio11.into_function::<FunctionPio0>();

    let pio_i2s_mclk_output = pio_file!("src/i2s.pio", select_program("mclk_output")).program;
    let pio_i2s_send_master = pio_file!("src/i2s.pio", select_program("i2s_out_master")).program;

    let (mut pio, sm0, sm1, _, _) = pac.PIO0.split(&mut pac.RESETS);
    let pio_i2s_mclk_output = pio.install(&pio_i2s_mclk_output).unwrap();
    let pio_i2s_send_master = pio.install(&pio_i2s_send_master).unwrap();

    let (mut sm0, _rx0, _tx0) = PIOBuilder::from_program(pio_i2s_mclk_output)
        .set_pins(i2s_sck_pin.id().num, 1)
        .clock_divisor_fixed_point(MCLK_CLOCKDIV_INT, MCLK_CLOCKDIV_FRAC) // TODO: hardcoded, should give a clock at LRCLK frequency * 256
        .build(sm0);

    let (mut sm1, _rx1, tx1) = PIOBuilder::from_program(pio_i2s_send_master)
        .out_pins(i2s_din_pin.id().num, 1)
        .side_set_pin_base(i2s_bck_pin.id().num)
        .clock_divisor_fixed_point(I2S_PIO_CLOCKDIV_INT, I2S_PIO_CLOCKDIV_FRAC)
        .out_shift_direction(ShiftDirection::Left)
        .autopull(true)
        .pull_threshold(16u8)
        .buffers(Buffers::OnlyTx)
        .build(sm1);

    sm0.set_pindirs([(i2s_sck_pin.id().num, PinDir::Output)]);
    sm0.start();
    sm1.set_pindirs([
        (i2s_din_pin.id().num, PinDir::Output),
        (i2s_lck_pin.id().num, PinDir::Output),
        (i2s_bck_pin.id().num, PinDir::Output),
    ]);
    sm1.start();

    let dma_channels = pac.DMA.split(&mut pac.RESETS);
    let i2s_tx_buf1 = singleton!(: [u32; BUFFER_SIZE*2] = [0; BUFFER_SIZE*2]).unwrap();
    let i2s_tx_buf2 = singleton!(: [u32; BUFFER_SIZE*2] = [0; BUFFER_SIZE*2]).unwrap();
    let i2s_dma_config =
        double_buffer::Config::new((dma_channels.ch0, dma_channels.ch1), i2s_tx_buf1, tx1);
    let i2s_tx_transfer = i2s_dma_config.start();
    let mut i2s_tx_transfer = i2s_tx_transfer.read_next(i2s_tx_buf2);

    delay.delay_ms(100);

    info!("Start Synth core.");

    let mut synth = SawtoothSynth::new(SawtoothSynthSettings {
        decay: U8F8::from_num(0.9),
    });

    let mut sample = 0i16;

    let mut warned = false;
    let mut ran_command = false;

    loop {
        ran_command = false;
        sio.fifo
            .read()
            .and_then(Command::deserialize)
            .inspect(|&command| {
                trace!("Running Synth command: {}", command);
                ran_command = true;
                synth.run_command(command)
            });

        if !warned && i2s_tx_transfer.is_done() {
            warn!("i2s transfer already done, probably late.");
            warned = true;
        }

        if ran_command && i2s_tx_transfer.is_done() {
            warn!("i2s transfer already done, late caused by command.");
        }

        let (next_tx_buf, next_tx_transfer) = i2s_tx_transfer.wait();
        for (i, e) in next_tx_buf.iter_mut().enumerate() {
            if i % 2 == 0 {
                sample = synth.next().to_bits();
                *e = sample as u32 / 2;
            } else {
                *e = sample as u32 / 2;
            }
            *e <<= 16;
        }

        i2s_tx_transfer = next_tx_transfer.read_next(next_tx_buf);
    }
}

static FIFO: Mutex<RefCell<Option<SioFifo>>> = Mutex::new(RefCell::new(None));
static ALARM: Mutex<RefCell<Option<Alarm1>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let mut sio = Sio::new(pac.SIO);

    let xosc = setup_xosc_blocking(pac.XOSC, EXTERNAL_XTAL_FREQ_HZ)
        .map_err(InitError::XoscErr)
        .ok()
        .unwrap();

    watchdog.enable_tick_generation((EXTERNAL_XTAL_FREQ_HZ.raw() / 1_000_000) as u8);

    let mut clocks = ClocksManager::new(pac.CLOCKS);

    {
        let pll_sys = setup_pll_blocking(
            pac.PLL_SYS,
            xosc.operating_frequency(),
            plls::SYS_PLL_CONFIG_307P2MHZ,
            &mut clocks,
            &mut pac.RESETS,
        )
        .map_err(InitError::PllError)
        .ok()
        .unwrap();

        let pll_usb = setup_pll_blocking(
            pac.PLL_USB,
            xosc.operating_frequency(),
            PLL_USB_48MHZ,
            &mut clocks,
            &mut pac.RESETS,
        )
        .map_err(InitError::PllError)
        .ok()
        .unwrap();

        clocks
            .reference_clock
            .configure_clock(&xosc, xosc.get_freq())
            .map_err(InitError::ClockError)
            .ok()
            .unwrap();

        clocks
            .system_clock
            .configure_clock(&pll_sys, pll_sys.get_freq())
            .map_err(InitError::ClockError)
            .ok()
            .unwrap();

        clocks
            .usb_clock
            .configure_clock(&pll_usb, pll_usb.get_freq())
            .map_err(InitError::ClockError)
            .ok()
            .unwrap();

        clocks
            .adc_clock
            .configure_clock(&pll_usb, pll_usb.get_freq())
            .map_err(InitError::ClockError)
            .ok()
            .unwrap();

        clocks
            .rtc_clock
            .configure_clock(&pll_usb, HertzU32::from_raw(46875u32))
            .map_err(InitError::ClockError)
            .ok()
            .unwrap();

        clocks
            .peripheral_clock
            .configure_clock(&clocks.system_clock, clocks.system_clock.freq())
            .map_err(InitError::ClockError)
            .ok()
            .unwrap();
    }

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Setup the other core
    let sys_freq = clocks.system_clock.freq().to_Hz();
    let mut mc = Multicore::new(&mut pac.PSM, &mut pac.PPB, &mut sio.fifo);
    let cores = mc.cores();
    let core1 = &mut cores[1];
    let _test = core1.spawn(unsafe { &mut CORE1_STACK.mem }, move || {
        synth_core(sys_freq)
    });

    let mut timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);
    let mut alarm = timer.alarm_1().unwrap();

    cortex_m::interrupt::free(move |cs| {
        FIFO.borrow(cs).replace(Some(sio.fifo));

        alarm
            .schedule(Duration::<u32, 1, 1000000>::millis(1))
            .unwrap();
        alarm.enable_interrupt();

        ALARM.borrow(cs).replace(Some(alarm));
    });

    let c_pin = pins.gpio4.into_pull_up_input();
    let cis_pin = pins.gpio5.into_pull_up_input();
    let d_pin = pins.gpio6.into_pull_up_input();
    let dis_pin = pins.gpio7.into_pull_up_input();
    let e_pin = pins.gpio12.into_pull_up_input();
    let f_pin = pins.gpio13.into_pull_up_input();
    let fis_pin = pins.gpio14.into_pull_up_input();
    let g_pin = pins.gpio15.into_pull_up_input();
    let gis_pin = pins.gpio19.into_pull_up_input();
    let a_pin = pins.gpio18.into_pull_up_input();
    let ais_pin = pins.gpio17.into_pull_up_input();
    let b_pin = pins.gpio16.into_pull_up_input();

    let fn0_pin = pins.gpio0.into_pull_up_input();
    let fn1_pin = pins.gpio1.into_pull_up_input();
    let fn2_pin = pins.gpio2.into_pull_up_input();
    let fn3_pin = pins.gpio3.into_pull_up_input();

    info!("Start I/O thread.");

    let octave = 4;
    let mut button_states = [false; 12];

    loop {
        let new_button_states = [
            c_pin.is_low().unwrap(),
            cis_pin.is_low().unwrap(),
            d_pin.is_low().unwrap(),
            dis_pin.is_low().unwrap(),
            e_pin.is_low().unwrap(),
            f_pin.is_low().unwrap(),
            fis_pin.is_low().unwrap(),
            g_pin.is_low().unwrap(),
            gis_pin.is_low().unwrap(),
            a_pin.is_low().unwrap(),
            ais_pin.is_low().unwrap(),
            b_pin.is_low().unwrap(),
        ];

        let command = if new_button_states[0] && !button_states[0] {
            Some(Command::Play(c!(octave), U8F8::from_num(1.0)))
        } else if new_button_states[1] && !button_states[1] {
            Some(Command::Play(cis!(octave), U8F8::from_num(1.0)))
        } else if new_button_states[2] && !button_states[2] {
            Some(Command::Play(d!(octave), U8F8::from_num(1.0)))
        } else if new_button_states[3] && !button_states[3] {
            Some(Command::Play(dis!(octave), U8F8::from_num(1.0)))
        } else if new_button_states[4] && !button_states[4] {
            Some(Command::Play(e!(octave), U8F8::from_num(1.0)))
        } else if new_button_states[5] && !button_states[5] {
            Some(Command::Play(f!(octave), U8F8::from_num(1.0)))
        } else if new_button_states[6] && !button_states[6] {
            Some(Command::Play(fis!(octave), U8F8::from_num(1.0)))
        } else if new_button_states[7] && !button_states[7] {
            Some(Command::Play(g!(octave), U8F8::from_num(1.0)))
        } else if new_button_states[8] && !button_states[8] {
            Some(Command::Play(gis!(octave), U8F8::from_num(1.0)))
        } else if new_button_states[9] && !button_states[9] {
            Some(Command::Play(a!(octave), U8F8::from_num(1.0)))
        } else if new_button_states[10] && !button_states[10] {
            Some(Command::Play(ais!(octave), U8F8::from_num(1.0)))
        } else if new_button_states[11] && !button_states[11] {
            Some(Command::Play(b!(octave), U8F8::from_num(1.0)))
        } else if button_states.iter().any(|&b| b) && new_button_states.iter().all(|b| !b) {
            Some(Command::Play(c!(0), U8F8::from_num(0.0)))
        } else {
            None
        };

        if let Some(command) = command {
            let command_serialized = command.serialize();

            cortex_m::interrupt::free(|cs| {
                let mut fifo = FIFO.borrow(cs).take().unwrap();
                fifo.write(command_serialized);
                FIFO.borrow(cs).replace(Some(fifo));
            });
        }

        button_states = new_button_states;
    }
}
