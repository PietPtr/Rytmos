#[macro_export]
macro_rules! setup_clocks {
    ($pac:ident, $clocks:ident, $config:expr) => {
        let xosc = setup_xosc_blocking($pac.XOSC, EXTERNAL_XTAL_FREQ_HZ)
            .map_err(InitError::XoscErr)
            .ok()
            .unwrap();

        {
            let pll_sys = setup_pll_blocking(
                $pac.PLL_SYS,
                xosc.operating_frequency(),
                $config,
                &mut $clocks,
                &mut $pac.RESETS,
            )
            .map_err(InitError::PllError)
            .ok()
            .unwrap();

            let pll_usb = setup_pll_blocking(
                $pac.PLL_USB,
                xosc.operating_frequency(),
                PLL_USB_48MHZ,
                &mut $clocks,
                &mut $pac.RESETS,
            )
            .map_err(InitError::PllError)
            .ok()
            .unwrap();

            $clocks
                .reference_clock
                .configure_clock(&xosc, xosc.get_freq())
                .map_err(InitError::ClockError)
                .ok()
                .unwrap();

            $clocks
                .system_clock
                .configure_clock(&pll_sys, pll_sys.get_freq())
                .map_err(InitError::ClockError)
                .ok()
                .unwrap();

            $clocks
                .usb_clock
                .configure_clock(&pll_usb, pll_usb.get_freq())
                .map_err(InitError::ClockError)
                .ok()
                .unwrap();

            $clocks
                .adc_clock
                .configure_clock(&pll_usb, pll_usb.get_freq())
                .map_err(InitError::ClockError)
                .ok()
                .unwrap();

            $clocks
                .rtc_clock
                .configure_clock(&pll_usb, HertzU32::from_raw(46875u32))
                .map_err(InitError::ClockError)
                .ok()
                .unwrap();

            $clocks
                .peripheral_clock
                .configure_clock(&$clocks.system_clock, $clocks.system_clock.freq())
                .map_err(InitError::ClockError)
                .ok()
                .unwrap();
        }
    };
}
