#![recursion_limit = "500"]
#![warn(clippy)]
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate yew;
extern crate http;
#[macro_use]
extern crate stdweb;
#[macro_use]
extern crate failure;

mod agents;
mod components;
mod context;
mod eos;
mod pages;
mod route;
mod services;
mod types;

pub use components::App as Model;