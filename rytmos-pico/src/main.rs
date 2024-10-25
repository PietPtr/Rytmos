#![no_std]
#![no_main]

#[link_section = ".boot2"]
#[no_mangle]
#[used]
pub static BOOT2_FIRMWARE: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

use cortex_m::singleton;
use defmt::*;
use defmt_rtt as _;
use fugit::HertzU32;
use panic_probe as _;
use pio_proc::pio_file;

use rp_pico::{
    entry,
    hal::{
        clocks::{Clock, ClockSource, ClocksManager, InitError},
        dma::{double_buffer, DMAExt},
        gpio::{self, FunctionPio0},
        multicore::{Multicore, Stack},
        pac,
        pio::{Buffers, PIOBuilder, PIOExt, PinDir, ShiftDirection},
        pll::{common_configs::PLL_USB_48MHZ, setup_pll_blocking},
        sio::Sio,
        watchdog::Watchdog,
        xosc::setup_xosc_blocking,
    },
};
use rytmos_engrave::*;
use rytmos_synth::{
    commands::Command,
    synth::{master::OvertoneAndMetronomeSynth, Synth},
};

mod plls;

const EXTERNAL_XTAL_FREQ_HZ: HertzU32 = HertzU32::from_raw(12_000_000u32);
const RP2040_CLOCK_HZ: HertzU32 = HertzU32::from_raw(307_200_000u32);

const SAMPLE_RATE: HertzU32 = HertzU32::from_raw(96_000u32);

const I2S_PIO_CLOCK_HZ: HertzU32 = HertzU32::from_raw(SAMPLE_RATE.raw() * 64u32 * 5u32);
const I2S_PIO_CLOCKDIV_INT: u16 = (RP2040_CLOCK_HZ.raw() / I2S_PIO_CLOCK_HZ.raw()) as u16;
const I2S_PIO_CLOCKDIV_FRAC: u8 = 0u8;

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

    let mclk_pin = pins.gpio8.into_function::<FunctionPio0>();
    let i2s_send_data_pin = pins.gpio9.into_function::<FunctionPio0>();
    let i2s_send_sclk_pin = pins.gpio10.into_function::<FunctionPio0>();
    let i2s_send_lrclk_pin = pins.gpio11.into_function::<FunctionPio0>();

    let pio_i2s_mclk_output = pio_file!("src/i2s.pio", select_program("mclk_output")).program;
    let pio_i2s_send_master = pio_file!("src/i2s.pio", select_program("i2s_send_master")).program;

    let (mut pio, sm0, sm1, _, _) = pac.PIO0.split(&mut pac.RESETS);
    let pio_i2s_mclk_output = pio.install(&pio_i2s_mclk_output).unwrap();
    let pio_i2s_send_master = pio.install(&pio_i2s_send_master).unwrap();

    let (mut sm0, _rx0, _tx0) = PIOBuilder::from_program(pio_i2s_mclk_output)
        .set_pins(mclk_pin.id().num, 1)
        .clock_divisor_fixed_point(I2S_PIO_CLOCKDIV_INT, I2S_PIO_CLOCKDIV_FRAC)
        .build(sm0);

    let (mut sm1, _rx1, tx1) = PIOBuilder::from_program(pio_i2s_send_master)
        .out_pins(i2s_send_data_pin.id().num, 1)
        .side_set_pin_base(i2s_send_sclk_pin.id().num)
        .clock_divisor_fixed_point(I2S_PIO_CLOCKDIV_INT, I2S_PIO_CLOCKDIV_FRAC)
        .out_shift_direction(ShiftDirection::Left)
        .autopull(true)
        .pull_threshold(32u8)
        .buffers(Buffers::OnlyTx)
        .build(sm1);

    sm0.set_pindirs([(mclk_pin.id().num, PinDir::Output)]);
    sm0.start();
    sm1.set_pindirs([
        (i2s_send_data_pin.id().num, PinDir::Output),
        (i2s_send_lrclk_pin.id().num, PinDir::Output),
        (i2s_send_sclk_pin.id().num, PinDir::Output),
    ]);

    let dma_channels = pac.DMA.split(&mut pac.RESETS);
    let i2s_tx_buf1 = singleton!(: [u32; BUFFER_SIZE*2] = [12345; BUFFER_SIZE*2]).unwrap();
    let i2s_tx_buf2 = singleton!(: [u32; BUFFER_SIZE*2] = [123; BUFFER_SIZE*2]).unwrap();
    let i2s_dma_config =
        double_buffer::Config::new((dma_channels.ch0, dma_channels.ch1), i2s_tx_buf1, tx1);
    let i2s_tx_transfer = i2s_dma_config.start();
    let mut i2s_tx_transfer = i2s_tx_transfer.read_next(i2s_tx_buf2);

    delay.delay_ms(100);

    info!("Start Synth core.");

    let mut synth = OvertoneAndMetronomeSynth::new();

    loop {
        sio.fifo
            .read()
            .and_then(Command::deserialize)
            .inspect(|&command| synth.run_command(command));

        if i2s_tx_transfer.is_done() {
            let (next_tx_buf, next_tx_transfer) = i2s_tx_transfer.wait();

            let sample = synth.next(); // TODO: i16 vs u32??

            for e in next_tx_buf.iter_mut() {
                *e = sample as u32;
            }

            i2s_tx_transfer = next_tx_transfer.read_next(next_tx_buf);
        }
    }
}

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

    let _pins = gpio::Pins::new(
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

    info!("Start I/O thread.");

    let commands = [
        Command::Play(c!(2), 155, 1),
        Command::Play(d!(2), 225, 1),
        Command::Play(e!(2), 240, 1),
        Command::Play(c!(2), 150, 2),
        Command::Play(c!(2), 0, 0),
        Command::Play(c!(2), 155, 1),
        Command::Play(d!(2), 225, 1),
        Command::Play(e!(2), 240, 1),
        Command::Play(c!(2), 150, 2),
        Command::Play(c!(2), 0, 0),
        Command::Play(e!(2), 240, 1),
        Command::Play(f!(2), 240, 1),
        Command::Play(g!(2), 200, 1),
        Command::Play(g!(2), 200, 1),
        Command::Play(c!(2), 0, 0),
        Command::Play(c!(2), 0, 0),
    ];

    let mut i = 0;
    loop {
        delay.delay_ms(800);

        sio.fifo.write(commands[i].serialize());

        if i > commands.len() {
            i = 0;
        } else {
            i += 1;
        }
    }
}
