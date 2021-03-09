mod roshambo;

use crate::game::Builder;

pub(crate) fn get() -> Vec<Box<dyn Builder>> {
    vec![roshambo::Builder::new()]
}
