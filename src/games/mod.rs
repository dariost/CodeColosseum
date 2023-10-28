#[macro_use]
mod util;

mod chess;
mod dama;
mod roshambo;
mod royalur;

use crate::game::Builder;

pub(crate) fn get() -> Vec<Box<dyn Builder>> {
    vec![roshambo::get(), royalur::get(), chess::get(), dama::get()]
}
