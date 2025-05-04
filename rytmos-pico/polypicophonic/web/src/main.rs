//! Run with:
//!
//! ```sh
//! dx serve --platform web
//! ```
#![allow(non_snake_case)]

use core::borrow::BorrowMut;
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
use web_sys::wasm_bindgen::JsCast;
use web_sys::{
    window, AudioContext, AudioWorkletNode, AudioWorkletNodeOptions, Request, RequestInit, Response,
};

fn keyboard_to_clavier(code: Code) -> Option<KeyId> {
    match code {
        Code::KeyZ => Some(KeyId::NoteC),
        Code::KeyS => Some(KeyId::NoteCis),
        Code::KeyX => Some(KeyId::NoteD),
        Code::KeyD => Some(KeyId::NoteDis),
        Code::KeyC => Some(KeyId::NoteE),
        Code::KeyV => Some(KeyId::NoteF),
        Code::KeyG => Some(KeyId::NoteFis),
        Code::KeyB => Some(KeyId::NoteG),
        Code::KeyH => Some(KeyId::NoteGis),
        Code::KeyN => Some(KeyId::NoteA),
        Code::KeyJ => Some(KeyId::NoteAis),
        Code::KeyM => Some(KeyId::NoteB),
        Code::KeyL => Some(KeyId::Fn0),
        Code::Semicolon => Some(KeyId::Fn1),
        Code::Comma => Some(KeyId::Fn2),
        Code::Period => Some(KeyId::Fn3),
        _ => None,
    }
}

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

    let mut key_signals_for_closure = key_signals.clone();
    let update_key_signals_down = move |event: KeyboardEvent| {
        let key_id = keyboard_to_clavier(event.data.code()).map(|id| id as usize);
        if let Some(key_id) = key_id {
            key_signals_for_closure[key_id].set(true);
        }
    };

    let mut key_signals_for_closure = key_signals.clone();
    let update_key_signals_up = move |event: KeyboardEvent| {
        let key_id = keyboard_to_clavier(event.data.code()).map(|id| id as usize);
        if let Some(key_id) = key_id {
            key_signals_for_closure[key_id].set(false);
        }
    };

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

    let mut play_audio = move || {
        drop(ctx_signal.borrow_mut().as_mut().unwrap().resume().unwrap());
    };

    rsx! {
        div {
            tabindex: 0,
            onkeydown: update_key_signals_down, // TODO: attach to window
            onkeyup: update_key_signals_up,

            document::Link { href: asset!("/assets/stylesheet.css"), rel: "stylesheet" }
            div {
                class: "header",
                h1 {
                    "Pico Piano"
                }

                button {
                    class: "start-button",
                    onclick: move |_| play_audio(),
                    "Start Audio Engine"
                }
            }

            // for (key, name) in key_signals.clone().into_iter().zip(keynames) {
            //     {pico_piano_button(key, name)}
            // }

            div {
                class: "all-buttons",
                div {
                    div {
                        class: "button-container keys",
                        div { class: "half-offset transparent" }
                        {pico_piano_button(key_signals[1], keynames[1], "key")}
                        {pico_piano_button(key_signals[3], keynames[3], "key")}
                        div { class: "key transparent" }
                        {pico_piano_button(key_signals[6], keynames[6], "key")}
                        {pico_piano_button(key_signals[8], keynames[8], "key")}
                        {pico_piano_button(key_signals[10], keynames[10], "key")}
                    }

                    div {
                        class: "button-container keys",
                        {pico_piano_button(key_signals[0], keynames[0], "key")}
                        {pico_piano_button(key_signals[2], keynames[2], "key")}
                        {pico_piano_button(key_signals[4], keynames[4], "key")}
                        {pico_piano_button(key_signals[5], keynames[5], "key")}
                        {pico_piano_button(key_signals[7], keynames[7], "key")}
                        {pico_piano_button(key_signals[9], keynames[9], "key")}
                        {pico_piano_button(key_signals[11], keynames[11], "key")}
                    }
                }

                div {
                    class: "all-fns",
                    div {
                        class: "button-container fns",
                        div { class: "fn-offset" }
                        {pico_piano_button(key_signals[12], keynames[12], "fn")}
                        {pico_piano_button(key_signals[13], keynames[13], "fn")}
                    }

                    div {
                        class: "button-container fns",
                        {pico_piano_button(key_signals[14], keynames[14], "fn")}
                        {pico_piano_button(key_signals[15], keynames[15], "fn")}
                    }
                }
            }

        }
    }
}

fn pico_piano_button(mut key: Signal<bool>, key_id: KeyId, class: &str) -> Element {
    let div_class = if class == "fn" { "fn-background" } else { "" };
    let active_class = if *key.read() {
        &format!("{class}-active")
    } else {
        ""
    };
    rsx! {
        div {
            class: div_class,
            button {
                class: "{class} {active_class}",
                onmousedown: move |_| key.set(true),
                onmouseup: move |_| key.set(false),
                onmouseleave: move |_| key.set(false),
                ""
            }
        }
    }
}

fn main() {
    dioxus::launch(app);
}
