pub mod badge;
pub mod button;
pub mod checkbox;
pub mod chip;
pub mod divider;
pub mod fab;
pub mod icon_button;
pub mod radio_button;
pub mod slider;
pub mod switch;

use std::rc::Rc;

use gpui::{AnyElement, App, Entity, Window};

use crate::systems::interaction::controller::InteractionController;

pub type ElementBuilder = Box<dyn Fn(&mut Window, &mut App) -> AnyElement>;
pub type ChangeHandler<T> = Rc<dyn Fn(T, &mut Window, &mut App)>;

pub fn sync_enabled(controller: &Entity<InteractionController>, enabled: bool, cx: &mut App) {
    let disabled = !enabled;
    if controller.read(cx).is_disabled() != disabled {
        controller.update(cx, |state, cx| state.set_disabled(disabled, cx));
    }
}
