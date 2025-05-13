//! Run with:
//!
//! ```sh
//! dx serve --platform web
//! ```
#![allow(non_snake_case)]

use core::borrow::BorrowMut;
use std::sync::OnceLock;
use std::time::Duration;
use std::{convert::TryFrom, iter::Iterator};

use dioxus::prelude::*;
use polypicophonic::{
    clavier::KeyId,
    interface::{sandbox::SandboxInterface, Interface},
    io::IO,
};
use polypicophonic_web::io::{WebFifo, WebKeys};
use wasm_bindgen_futures::JsFuture;
use web_sys::js_sys::Array;
use web_sys::wasm_bindgen::prelude::Closure;
use web_sys::wasm_bindgen::JsCast;
use web_sys::{
    window, AudioContext, AudioWorkletNode, AudioWorkletNodeOptions, Request, RequestInit, Response,
};

fn keyboard_to_clavier_str(code: &str) -> Option<KeyId> {
    match code {
        "KeyZ" => Some(KeyId::NoteC),
        "KeyS" => Some(KeyId::NoteCis),
        "KeyX" => Some(KeyId::NoteD),
        "KeyD" => Some(KeyId::NoteDis),
        "KeyC" => Some(KeyId::NoteE),
        "KeyV" => Some(KeyId::NoteF),
        "KeyG" => Some(KeyId::NoteFis),
        "KeyB" => Some(KeyId::NoteG),
        "KeyH" => Some(KeyId::NoteGis),
        "KeyN" => Some(KeyId::NoteA),
        "KeyJ" => Some(KeyId::NoteAis),
        "KeyM" => Some(KeyId::NoteB),
        "KeyL" => Some(KeyId::Fn0),
        "Semicolon" => Some(KeyId::Fn1),
        "Comma" => Some(KeyId::Fn2),
        "Period" => Some(KeyId::Fn3),
        _ => None,
    }
}

// TODO: has to be manually kept up to date with keyboard_to_clavier_str...
fn key_id_to_keyboard_help(key_id: KeyId) -> String {
    match key_id {
        KeyId::NoteC => "Z",
        KeyId::NoteCis => "S",
        KeyId::NoteD => "X",
        KeyId::NoteDis => "D",
        KeyId::NoteE => "C",
        KeyId::NoteF => "V",
        KeyId::NoteFis => "G",
        KeyId::NoteG => "B",
        KeyId::NoteGis => "H",
        KeyId::NoteA => "N",
        KeyId::NoteAis => "J",
        KeyId::NoteB => "M",
        KeyId::Fn0 => "L",
        KeyId::Fn1 => ";",
        KeyId::Fn2 => ",",
        KeyId::Fn3 => ".",
    }
    .to_string()
}

static ONCE: OnceLock<()> = OnceLock::new();

fn app() -> Element {
    let mut ctx_signal: Signal<Option<AudioContext>> = use_signal(|| None);
    let mut node_signal: Signal<Option<AudioWorkletNode>> = use_signal(|| None);

    let keynames: Vec<KeyId> = (0..16)
        .map(|i| (KeyId::try_from(i).unwrap()))
        .collect::<Vec<_>>();

    let key_signals = (0..16).map(|_| use_signal(|| false)).collect::<Vec<_>>();

    use_future({
        let key_signals = key_signals.clone();
        move || {
            let key_signals = key_signals.clone();
            async move {
                let io = IO {
                    fifo: WebFifo::new(node_signal),
                    clavier: WebKeys::new(key_signals),
                };

                let mut interface = SandboxInterface::new(io);

                loop {
                    // for _ in 0..10 {
                    interface.run();

                    async_std::task::sleep(Duration::from_millis(10)).await
                }
            }
        }
    });

    use_future(move || async move {
        let ctx = AudioContext::new().unwrap();

        JsFuture::from(
            ctx.audio_worklet()
                .unwrap()
                .add_module(&asset!("/assets/wasm_audio.js").to_string())
                .unwrap(),
        )
        .await
        .unwrap();

        let options = RequestInit::new();
        options.set_method("GET");
        let request = Request::new_with_str_and_init(
            &asset!("/assets/wasm_audio_bg.wasm").to_string(),
            &options,
        )
        .unwrap();

        let window = window().unwrap();
        let response = JsFuture::from(window.fetch_with_request(&request))
            .await
            .unwrap()
            .unchecked_into::<Response>();

        let array_buffer = JsFuture::from(response.array_buffer().unwrap())
            .await
            .unwrap();

        let options = AudioWorkletNodeOptions::new();
        options.set_processor_options(Some(&Array::of1(&array_buffer)));

        let node = AudioWorkletNode::new_with_options(&ctx, "my-processor", &options).unwrap();
        node.connect_with_audio_node(&ctx.destination()).unwrap();

        node_signal.set(Some(node));
        ctx_signal.set(Some(ctx));
    });

    let mut is_audio_initialized = use_signal(|| false);

    let mut initialize_audio = move || {
        if !is_audio_initialized() {
            drop(ctx_signal.borrow_mut().as_mut().unwrap().resume().unwrap());
        }

        is_audio_initialized.set(true);
    };

    let mut shift_signal = use_signal(|| false);

    ONCE.get_or_init({
        let key_signals = key_signals.clone();
        move || {
            let mut key_signals = key_signals.clone();
            let mut key_signals_up = key_signals.clone();
            let keydown_closure: Closure<dyn FnMut(web_sys::KeyboardEvent)> =
                Closure::new(move |event: web_sys::KeyboardEvent| {
                    if !event.shift_key()
                        && !event.ctrl_key()
                        && !event.alt_key()
                        && !event.meta_key()
                    {
                        initialize_audio();
                    }

                    let key_id = keyboard_to_clavier_str(&event.code()).map(|id| id as usize);
                    if let Some(key_id) = key_id {
                        key_signals[key_id].set(true);
                    }

                    match event.code().as_str() {
                        "ShiftLeft" | "ShiftRight" => shift_signal.set(true),
                        _ => {}
                    }
                });

            let keyup_closure: Closure<dyn FnMut(web_sys::KeyboardEvent)> =
                Closure::new(move |event: web_sys::KeyboardEvent| {
                    let key_id = keyboard_to_clavier_str(&event.code()).map(|id| id as usize);
                    if let Some(key_id) = key_id {
                        key_signals_up[key_id].set(false);
                    }

                    match event.code().as_str() {
                        "ShiftLeft" | "ShiftRight" => shift_signal.set(false),
                        _ => {}
                    }
                });

            let mousedown_closure: Closure<dyn FnMut(web_sys::MouseEvent)> =
                Closure::new(move |_| {
                    initialize_audio();
                });

            let window = window().unwrap();
            window
                .add_event_listener_with_callback(
                    "keydown",
                    keydown_closure.as_ref().unchecked_ref(),
                )
                .unwrap();
            window
                .add_event_listener_with_callback(
                    "mousedown",
                    mousedown_closure.as_ref().unchecked_ref(),
                )
                .unwrap();
            window
                .add_event_listener_with_callback("keyup", keyup_closure.as_ref().unchecked_ref())
                .unwrap();

            keydown_closure.forget();
            keyup_closure.forget();
            mousedown_closure.forget();
        }
    });

    rsx! {
        div {
            class: "content",
            tabindex: 0,

            document::Link { href: asset!("/assets/stylesheet.css"), rel: "stylesheet" }
            document::Link { rel: "icon", type: "image/png", href: asset!("/assets/icon.png") }
            document::Title { "Pico Piano" }

            div {
                class: "header",
                h1 {
                    "Pico Piano"
                }

                if !is_audio_initialized() {
                    button {
                        class: "start-button",
                        onclick: move |_| initialize_audio(),
                        "Start Audio Engine"
                    }
                }
            }

            div {
                class: "all-buttons",
                div {
                    div {
                        class: "button-container keys",
                        div { class: "half-offset transparent" }
                        {pico_piano_button(shift_signal, key_signals[1], keynames[1], "key")}
                        {pico_piano_button(shift_signal, key_signals[3], keynames[3], "key")}
                        div { class: "key transparent" }
                        {pico_piano_button(shift_signal, key_signals[6], keynames[6], "key")}
                        {pico_piano_button(shift_signal, key_signals[8], keynames[8], "key")}
                        {pico_piano_button(shift_signal, key_signals[10], keynames[10], "key")}
                    }

                    div {
                        class: "button-container keys",
                        {pico_piano_button(shift_signal, key_signals[0], keynames[0], "key")}
                        {pico_piano_button(shift_signal, key_signals[2], keynames[2], "key")}
                        {pico_piano_button(shift_signal, key_signals[4], keynames[4], "key")}
                        {pico_piano_button(shift_signal, key_signals[5], keynames[5], "key")}
                        {pico_piano_button(shift_signal, key_signals[7], keynames[7], "key")}
                        {pico_piano_button(shift_signal, key_signals[9], keynames[9], "key")}
                        {pico_piano_button(shift_signal, key_signals[11], keynames[11], "key")}
                    }
                }

                div {
                    class: "all-fns",
                    div {
                        class: "button-container fns",
                        div { class: "fn-offset" }
                        {pico_piano_button(shift_signal, key_signals[12], keynames[12], "fn")}
                        {pico_piano_button(shift_signal, key_signals[13], keynames[13], "fn")}
                    }

                    div {
                        class: "button-container fns",
                        {pico_piano_button(shift_signal, key_signals[14], keynames[14], "fn")}
                        {pico_piano_button(shift_signal, key_signals[15], keynames[15], "fn")}
                    }
                }

            }

            p {
                class: "helptext",
                "(hold shift for hotkeys)"
            }
        }
    }
}

fn pico_piano_button(
    shift_signal: Signal<bool>,
    mut key: Signal<bool>,
    key_id: KeyId,
    class: &str,
) -> Element {
    let div_class = if class == "fn" { "fn-background" } else { "" };
    let active_class = if key() {
        &format!("{class}-active")
    } else {
        ""
    };
    let button_text = if shift_signal() {
        &key_id_to_keyboard_help(key_id)
    } else {
        ""
    };
    rsx! {
        div {
            class: div_class,
            button {
                class: "{class} {active_class} pico-button",
                onmousedown: move |_| key.set(true),
                ontouchstart: move |_| key.set(true),
                onmouseup: move |_| key.set(false),
                ontouchend: move |_| key.set(false),
                onmouseleave: move |_| key.set(false),
                "{button_text}"
            }
        }
    }
}

fn main() {
    dioxus::launch(app);
}
