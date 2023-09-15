#[macro_use]
mod util;

mod roshambo;
mod royalur;
mod chess;

use crate::game::Builder;

pub(crate) fn get() -> Vec<Box<dyn Builder>> {
    vec![roshambo::get(), royalur::get(), chess::get()]
}
