#![no_std]
#![no_main]

#[link_section = ".boot2"]
#[no_mangle]
#[used]
pub static BOOT2_FIRMWARE: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

use cortex_m::{asm, singleton};
#[allow(unused_imports)]
use defmt::{error, info, warn};
use defmt_rtt as _;
use drum_machine::{
    io::DrumIO,
    sequencer::{self, Sequence, SequenceTimeSignature, Sequencer},
};
use drum_machine_bsp::{
    entry,
    hal::{
        clocks::{Clock, ClockSource, ClocksManager},
        dma::{double_buffer, DMAExt},
        gpio::{self, FunctionPio0},
        multicore::{Multicore, Stack},
        pio::{Buffers, PIOBuilder, PIOExt, PinDir, ShiftDirection},
        pll::{common_configs::PLL_USB_48MHZ, setup_pll_blocking, PLLConfig},
        sio::Sio,
        watchdog::Watchdog,
        xosc::setup_xosc_blocking,
        Adc, Timer,
    },
    pac, Pins,
};
use fixed::types::U4F4;
use fugit::HertzU32;
use panic_probe as _;

use common::consts::*;
use rytmos_synth::commands::Command;
use rytmos_synth::synth::drum::DrumSynth;
use rytmos_synth::synth::drum::DrumSynthSettings;
use rytmos_synth::synth::Synth;

#[allow(dead_code)]
pub const SYS_PLL_CONFIG_307P2MHZ: PLLConfig = PLLConfig {
    vco_freq: HertzU32::MHz(1536),
    refdiv: 1,
    post_div1: 5,
    post_div2: 2,
};

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut sio = Sio::new(pac.SIO);

    let mut watchdog = Watchdog::new(pac.WATCHDOG);

    // let clocks = clocks::init_clocks_and_plls(
    //     12_000_000u32,
    //     pac.XOSC,
    //     pac.CLOCKS,
    //     pac.PLL_SYS,
    //     pac.PLL_USB,
    //     &mut pac.RESETS,
    //     &mut watchdog,
    // )
    // .unwrap();

    let mut clocks = ClocksManager::new(pac.CLOCKS);

    let xosc = setup_xosc_blocking(pac.XOSC, EXTERNAL_XTAL_FREQ_HZ).unwrap();

    watchdog.enable_tick_generation((12_000_000u32 / 1_000_000) as u8);

    {
        let pll_sys = setup_pll_blocking(
            pac.PLL_SYS,
            xosc.operating_frequency(),
            SYS_PLL_CONFIG_307P2MHZ,
            &mut clocks,
            &mut pac.RESETS,
        )
        .unwrap();

        let pll_usb = setup_pll_blocking(
            pac.PLL_USB,
            xosc.operating_frequency(),
            PLL_USB_48MHZ,
            &mut clocks,
            &mut pac.RESETS,
        )
        .unwrap();

        clocks
            .reference_clock
            .configure_clock(&xosc, xosc.get_freq())
            .unwrap();

        clocks
            .system_clock
            .configure_clock(&pll_sys, pll_sys.get_freq())
            .unwrap();

        clocks
            .usb_clock
            .configure_clock(&pll_usb, pll_usb.get_freq())
            .unwrap();

        clocks
            .adc_clock
            .configure_clock(&pll_usb, pll_usb.get_freq())
            .unwrap();

        clocks
            .rtc_clock
            .configure_clock(&pll_usb, HertzU32::from_raw(46875u32))
            .unwrap();

        clocks
            .peripheral_clock
            .configure_clock(&clocks.system_clock, clocks.system_clock.freq())
            .unwrap();
    }

    let mut _delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());
    let timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    let pins = Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    info!("Initializing other core.");
    let sys_freq = clocks.system_clock.freq().to_Hz();
    let mut mc = Multicore::new(&mut pac.PSM, &mut pac.PPB, &mut sio.fifo);
    let cores = mc.cores();
    let core1 = &mut cores[1];
    let _ = core1.spawn(unsafe { CORE1_STACK.take().unwrap() }, move || {
        synth_core(sys_freq)
    });

    info!("initializing I/O");

    let mut io = DrumIO::new(pins, Adc::new(pac.ADC, &mut pac.RESETS));

    info!("I/O initialized :)");

    let mut sequencer = Sequencer::new(io.update());

    loop {
        let start = timer.get_counter();
        let io_state = io.update();

        sequencer.change_sequence(Sequence {
            hat: sequencer::SingleSampleSequence {
                subdivs: io_state.hat,
                velocity: U4F4::from_bits((io_state.volume[0] >> 8) as u8),
            },
            snare: sequencer::SingleSampleSequence {
                subdivs: io_state.snare,
                velocity: U4F4::from_bits((io_state.volume[1] >> 8) as u8),
            },
            kick: sequencer::SingleSampleSequence {
                subdivs: io_state.kick,
                velocity: U4F4::from_bits((io_state.volume[2] >> 8) as u8),
            },
        });

        if !io_state.settings.leds_enabled || !sequencer.is_playing() {
            io.disable_led();
        } else {
            io.led_index(sequencer.current_subdivision());
        }

        if io_state.settings.play_or_pause {
            if io_state.settings.countoff_at_play {
                sequencer.play_with_countoff();
            } else {
                sequencer.play()
            }
        } else {
            sequencer.stop();
            io.disable_led();
        }

        sequencer.cymbal_every_four_measures = io_state.settings.cymbal_every_four_measures;
        sequencer.time_signature = if io_state.settings.time_signature {
            SequenceTimeSignature::FourFour
        } else {
            SequenceTimeSignature::TwelveEight
        };

        const MIN_BEATS_PER_KILOMINUTE: u32 = 10_000; // = 10 BPM
        const MAX_BEATS_PER_KILOMINUTE: u32 = 200_000;
        const BEATS_PER_KLIOMINUTE_MULTIPLIER: u32 =
            (MAX_BEATS_PER_KILOMINUTE - MIN_BEATS_PER_KILOMINUTE) / 4096;

        let bpkm = MIN_BEATS_PER_KILOMINUTE + io_state.bpm as u32 * BEATS_PER_KLIOMINUTE_MULTIPLIER;
        let microseconds_per_beat = 60_000_000_000u64 / bpkm as u64;
        let microseconds_per_interval = match sequencer.time_signature {
            SequenceTimeSignature::FourFour => microseconds_per_beat / 4,
            SequenceTimeSignature::TwelveEight => microseconds_per_beat / 3,
        };

        loop {
            let end = timer.get_counter();
            let time_taken_us = (end - start).to_micros();

            if time_taken_us > microseconds_per_interval {
                break;
            }
        }

        let _subdiv = sequencer.current_subdivision();
        let commands = sequencer.next_subdivision();
        for command in commands {
            sio.fifo.write(command.serialize());
            // info!("{} | command={}", _subdiv, command);
        }
    }
}

static mut CORE1_STACK: Stack<4096> = Stack::new();

fn synth_core(_sys_freq: u32) -> ! {
    let mut pac = unsafe { pac::Peripherals::steal() };
    let _core = unsafe { pac::CorePeripherals::steal() };
    let mut sio = Sio::new(pac.SIO);
    let pins = gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let i2s_sck_pin = pins.gpio12.into_function::<FunctionPio0>();
    let i2s_din_pin = pins.gpio13.into_function::<FunctionPio0>();
    let i2s_bck_pin = pins.gpio14.into_function::<FunctionPio0>();
    let i2s_lck_pin = pins.gpio15.into_function::<FunctionPio0>();

    let (pio_i2s_mclk_output, pio_i2s_send_master) = common::pio::i2s_programs();

    let (mut pio, sm0, sm1, _, _) = pac.PIO0.split(&mut pac.RESETS);
    let pio_i2s_mclk_output = pio.install(&pio_i2s_mclk_output).unwrap();
    let pio_i2s_send_master = pio.install(&pio_i2s_send_master).unwrap();

    let (mut sm0, _rx0, _tx0) = PIOBuilder::from_installed_program(pio_i2s_mclk_output)
        .set_pins(i2s_sck_pin.id().num, 1)
        .clock_divisor_fixed_point(MCLK_CLOCKDIV_INT, MCLK_CLOCKDIV_FRAC)
        .build(sm0);

    let (mut sm1, _rx1, tx1) = PIOBuilder::from_installed_program(pio_i2s_send_master)
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

    for _ in 0..1000 {
        asm::nop();
    }

    info!("Start Synth core.");

    let mut synth = DrumSynth::make(0, DrumSynthSettings {});

    loop {
        sio.fifo
            .read()
            .and_then(Command::deserialize)
            .inspect(|&command| synth.run_command(command));

        let mut sample = 0;
        let (next_tx_buf, next_tx_transfer) = i2s_tx_transfer.wait();
        for (i, e) in next_tx_buf.iter_mut().enumerate() {
            if i % 2 == 0 {
                sample = synth.next().to_bits();
                *e = (sample as u32) >> 4;
            } else {
                *e = (sample as u32) >> 4;
            }
            *e <<= 16;
        }

        i2s_tx_transfer = next_tx_transfer.read_next(next_tx_buf);
    }
}
