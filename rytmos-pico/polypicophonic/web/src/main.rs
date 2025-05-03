//! Run with:
//!
//! ```sh
//! dx serve --platform web
//! ```

#![allow(non_snake_case, unused)]
use std::thread;

use dioxus::prelude::*;
use polypicophonic::interface::chordloops::ChordLoopInterface;
use serde::{Deserialize, Serialize};

fn app() -> Element {
    let mut count = use_signal(|| 0);
    let mut text = use_signal(|| "...".to_string());

    let mut is_c_down = use_signal(|| false);

    rsx! {
        document::Link { href: asset!("/assets/stylesheet.css"), rel: "stylesheet" }
        h1 { "Pico Piano {is_c_down}" }
        button { onmousedown: move |_| is_c_down.set(true), onmouseup: move |_| {is_c_down.set(false); tracing::info!("uhhh")}, "C" }
    }
}

fn main() {
    dioxus::launch(app);
}
