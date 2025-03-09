#![no_std]
#![no_main]

#[link_section = ".boot2"]
#[no_mangle]
#[used]
pub static BOOT2_FIRMWARE: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

#[allow(unused_imports)]
use defmt::{error, info, warn};
use defmt_rtt as _;
use panic_probe as _;

use drum_machine_bsp::entry;

#[entry]
fn main() -> ! {
    // TODO: this
    loop {}
}
