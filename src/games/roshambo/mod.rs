mod bot;
mod builder;
mod instance;

use crate::game::Builder;

pub(crate) fn get() -> Box<dyn Builder> {
    builder::Builder::new()
}
