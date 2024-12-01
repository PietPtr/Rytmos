use pio::Program;
use pio_proc::pio_file;

pub fn i2s_programs() -> (Program<32>, Program<32>) {
    let pio_i2s_mclk_output = pio_file!("src/i2s.pio", select_program("mclk_output")).program;
    let pio_i2s_send_master = pio_file!("src/i2s.pio", select_program("i2s_out_master")).program;

    (pio_i2s_mclk_output, pio_i2s_send_master)
}
