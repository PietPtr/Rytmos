#![no_std]
#![no_main]

#[link_section = ".boot2"]
#[no_mangle]
#[used]
pub static BOOT2_FIRMWARE: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

use core::cell::RefCell;

use cortex_m::{interrupt::Mutex, singleton};
#[allow(unused_imports)]
use defmt::{error, info, warn};
use defmt_rtt as _;
use embedded_hal::digital::v2::InputPin;
use fixed::types::I1F15;
use fixed::types::U4F4;
use fugit::Duration;
use fugit::HertzU32;
use heapless::Vec;
use panic_probe as _;
use pio_proc::pio_file;
use polypicophonic::clavier::Clavier;
use polypicophonic::interface::sandbox::SandboxInterface;
use polypicophonic::interface::Interface;
use polypicophonic::interface::PicoPianoHardware;
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

use common::consts::*;
use common::debouncer::Debouncer;
use rytmos_engrave::staff::Note;
use rytmos_engrave::{a, ais, b, c, cis, d, dis, e, f, fis, g, gis};
use rytmos_synth::commands::CommandMessage;
use rytmos_synth::effect::linear_decay::LinearDecay;
use rytmos_synth::effect::linear_decay::LinearDecaySettings;
use rytmos_synth::effect::lpf::LowPassFilter;
use rytmos_synth::effect::lpf::LowPassFilterSettings;
use rytmos_synth::synth::composed::polyphonic::PolyphonicSynth;
use rytmos_synth::synth::composed::synth_with_effects::SynthWithEffect;
use rytmos_synth::synth::composed::synth_with_effects::SynthWithEffectSettings;
use rytmos_synth::synth::sine::SineSynth;
use rytmos_synth::synth::sine::SineSynthSettings;
use rytmos_synth::{commands::Command, synth::Synth};

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

    let i2s_sck_pin = pins.gpio12.into_function::<FunctionPio0>();
    let i2s_din_pin = pins.gpio13.into_function::<FunctionPio0>();
    let i2s_bck_pin = pins.gpio14.into_function::<FunctionPio0>();
    let i2s_lck_pin = pins.gpio15.into_function::<FunctionPio0>();

    let pio_i2s_mclk_output = pio_file!("src/i2s.pio", select_program("mclk_output")).program;
    let pio_i2s_send_master = pio_file!("src/i2s.pio", select_program("i2s_out_master")).program;

    let (mut pio, sm0, sm1, _, _) = pac.PIO0.split(&mut pac.RESETS);
    let pio_i2s_mclk_output = pio.install(&pio_i2s_mclk_output).unwrap();
    let pio_i2s_send_master = pio.install(&pio_i2s_send_master).unwrap();

    let (mut sm0, _rx0, _tx0) = PIOBuilder::from_program(pio_i2s_mclk_output)
        .set_pins(i2s_sck_pin.id().num, 1)
        .clock_divisor_fixed_point(MCLK_CLOCKDIV_INT, MCLK_CLOCKDIV_FRAC)
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

    // type Synth = SynthWithEffect<SynthWithEffect<SineSynth, LinearDecay>, LowPassFilter>;
    type WaveSynth = SineSynth;
    type Synth = SynthWithEffect<SynthWithEffect<WaveSynth, LinearDecay>, LowPassFilter>;

    let settings =
        SynthWithEffectSettings::<SynthWithEffect<WaveSynth, LinearDecay>, LowPassFilter> {
            synth: SynthWithEffectSettings::<WaveSynth, LinearDecay> {
                synth: SineSynthSettings::default(),
                effect: LinearDecaySettings {
                    decay: I1F15::from_num(0.0005),
                    decay_every: 32,
                },
            },
            effect: LowPassFilterSettings {
                alpha: I1F15::from_num(0.05),
            },
        };

    let mut synth = PolyphonicSynth::<4, Synth>::make(0, settings);

    let mut sample = 0i16;

    let mut warned = false;

    loop {
        sio.fifo
            .read()
            .and_then(Command::deserialize)
            .inspect(|&command| synth.run_command(command));

        if !warned && i2s_tx_transfer.is_done() {
            warn!("i2s transfer already done, probably late.");
            warned = true;
        }

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

static FIFO: Mutex<RefCell<Option<SioFifo>>> = Mutex::new(RefCell::new(None));
static ALARM: Mutex<RefCell<Option<Alarm1>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let mut sio = Sio::new(pac.SIO);

    watchdog.enable_tick_generation((EXTERNAL_XTAL_FREQ_HZ.raw() / 1_000_000) as u8);

    let mut clocks = ClocksManager::new(pac.CLOCKS);

    // common::setup_clocks!(pac, clocks, common::plls::SYS_PLL_CONFIG_307P2MHZ);

    let mut _delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = rp_pico::Pins::new(
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
    #[allow(static_mut_refs)]
    let _test = core1.spawn(unsafe { &mut CORE1_STACK.mem }, move || {
        synth_core(sys_freq)
    });

    let mut timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);
    let mut alarm = timer.alarm_1().unwrap();

    {
        // TODO: do at boot check which interface should be created
    }

    let hw = PicoPianoHardware {
        fifo: sio.fifo,
        clavier: Clavier::new(pins),
    };

    let interface = SandboxInterface::new(hw);

    info!("Start interface thread.");

    interface.start();
}
