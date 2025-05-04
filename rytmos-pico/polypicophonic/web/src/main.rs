//! Run with:
//!
//! ```sh
//! dx serve --platform web
//! ```
#![allow(non_snake_case)]

pub mod io;

use std::time::Duration;
use std::{convert::TryFrom, iter::Iterator};

use dioxus::prelude::*;
use io::{WebFifo, WebKeys};
use polypicophonic::{
    clavier::KeyId,
    interface::{sandbox::SandboxInterface, Interface},
    io::IO,
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
    let key_signals_for_closure = key_signals.clone();

    use_future(move || {
        let key_signals_for_closure = key_signals_for_closure.clone();
        async move {
            let io = IO {
                fifo: WebFifo {},
                clavier: WebKeys::new(key_signals_for_closure),
            };

            let mut interface = SandboxInterface::new(io);

            loop {
                // for _ in 0..10 {
                interface.run();

                async_std::task::sleep(Duration::from_millis(10)).await
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

    rsx! {
        div {
            tabindex: 0,
            onkeydown: update_key_signals_down,
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

        }
    }
}

fn main() {
    dioxus::launch(app);
}
