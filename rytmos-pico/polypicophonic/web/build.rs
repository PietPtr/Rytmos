use std::process::Command;

fn main() {
    Command::new("cp")
        .args(["../wasm-audio/pkg/wasm_audio.js", "assets/"])
        .status()
        .unwrap();

    Command::new("cp")
        .args(["../wasm-audio/pkg/wasm_audio_bg.wasm", "assets/"])
        .status()
        .unwrap();
}
