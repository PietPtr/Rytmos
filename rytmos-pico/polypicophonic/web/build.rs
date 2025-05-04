use std::process::Command;

fn main() {
    let status_js = Command::new("cp")
        .args(["../wasm-audio/pkg/wasm_audio.js", "assets/"])
        .status()
        .unwrap();

    if !status_js.success() {
        panic!("cp failed for wasm_audio.js");
    }

    let status_wasm = Command::new("cp")
        .args(["../wasm-audio/pkg/wasm_audio_bg.wasm", "assets/"])
        .status()
        .unwrap();

    if !status_wasm.success() {
        panic!("cp failed for wasm_audio_bg.wasm");
    }
}
