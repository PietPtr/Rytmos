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
    window, AudioContext, AudioWorkletNode, AudioWorkletNodeOptions, OscillatorType, Request,
    RequestInit, Response,
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
        Code::Period => Some(KeyId::Fn2),
        Code::Slash => Some(KeyId::Fn3),
        _ => None,
    }
}

fn app() -> Element {
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
                    fifo: WebFifo {},
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

    let mut ctx_signal: Signal<Option<AudioContext>> = use_signal(|| None);

    use_future(move || async move {
        let ctx = AudioContext::new().unwrap();

        let synth = ctx.create_oscillator().unwrap();

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

        let my_node = AudioWorkletNode::new_with_options(&ctx, "my-processor", &options).unwrap();
        my_node.connect_with_audio_node(&ctx.destination()).unwrap();

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
            onmousedown: move |_| tracing::info!("muis"),

            document::Link { href: asset!("/assets/stylesheet.css"), rel: "stylesheet" }
            h1 {
                "C: {key_signals[0]}"
            }

            for (mut key, name) in key_signals.clone().into_iter().zip(keynames) {
                button {
                    onmousedown: move |_| key.set(true),
                    onmouseup: move |_| key.set(false),
                    onmouseleave: move |_| key.set(false),
                    "{name:?}"
                }
            }

            button {
                onclick: move |_| play_audio(),
                "start"
            }
        }
    }
}

fn main() {
    dioxus::launch(app);
}
