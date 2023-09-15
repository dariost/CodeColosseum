#[allow(warnings)]
mod bot;
#[allow(warnings)]
mod color;
#[allow(warnings)]
mod board;
#[allow(warnings)]
mod chess_move;
#[allow(warnings)]
mod builder;
#[allow(warnings)]
mod instance;

use crate::game::Builder;

pub(crate) fn get() -> Box<dyn Builder> {
    builder::Builder::new()
}
