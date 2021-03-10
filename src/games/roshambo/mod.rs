mod builder;

use crate::game::Builder;

pub(crate) fn get() -> Box<dyn Builder> {
    builder::Builder::new()
}
