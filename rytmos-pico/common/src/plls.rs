use fugit::HertzU32;
use rp_pico::hal::pll::PLLConfig;

#[allow(dead_code)]
pub const SYS_PLL_CONFIG_76P8MHZ: PLLConfig = PLLConfig {
    vco_freq: HertzU32::MHz(1536),
    refdiv: 1,
    post_div1: 5,
    post_div2: 4,
};

#[allow(dead_code)]
pub const SYS_PLL_CONFIG_153P6MHZ: PLLConfig = PLLConfig {
    vco_freq: HertzU32::MHz(1536),
    refdiv: 1,
    post_div1: 5,
    post_div2: 2,
};

#[allow(dead_code)]
pub const SYS_PLL_CONFIG_230P4MHZ: PLLConfig = PLLConfig {
    vco_freq: HertzU32::MHz(1152),
    refdiv: 1,
    post_div1: 5,
    post_div2: 1,
};

#[allow(dead_code)]
pub const SYS_PLL_CONFIG_307P2MHZ: PLLConfig = PLLConfig {
    vco_freq: HertzU32::MHz(1536),
    refdiv: 1,
    post_div1: 5,
    post_div2: 1,
};

#[allow(dead_code)]
pub const SYS_PLL_CONFIG_384MHZ: PLLConfig = PLLConfig {
    vco_freq: HertzU32::MHz(1536),
    refdiv: 1,
    post_div1: 4,
    post_div2: 1,
};
