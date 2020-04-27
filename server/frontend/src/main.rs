#![deny(warnings)]
// yew uses a lot of macros...
#![recursion_limit="1024"]

extern crate yew;
extern crate common;
#[macro_use]
extern crate stdweb;
extern crate nalgebra as na;
#[macro_use]
extern crate lazy_static;
extern crate chrono;
extern crate palette;
extern crate num;
extern crate failure;

#[macro_use]
mod util;
mod canvas;
mod components;

use components::root::RootComponent;

fn main() {
    yew::start_app::<RootComponent>();
}

