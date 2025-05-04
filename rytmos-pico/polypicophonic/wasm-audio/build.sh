wasm-pack build --target web

# TODO: lmao
cat assets/TextDecoder.js pkg/wasm_audio.js assets/worklet.js >> pkg/wasm_audio.js.tmp
mv pkg/wasm_audio.js.tmp pkg/wasm_audio.js

# If a directory is provided, copy the build products there
if [ -n "$1" ]; then
    cp pkg/wasm_audio_bg.wasm pkg/wasm_audio.js "$1"
fi
