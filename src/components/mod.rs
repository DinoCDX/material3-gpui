pub mod badge;
pub mod button;
pub mod checkbox;
pub mod chip;
pub mod divider;
pub mod fab;
pub mod icon;
pub mod icon_button;
pub mod radio_button;
pub mod slider;
pub mod switch;
pub mod text_field;

use std::rc::Rc;

use gpui::{AnyElement, App, Entity, IntoElement, Window};

use crate::systems::interaction::controller::InteractionController;
use icon::{Icon, IconKind};

pub type ElementBuilder = Box<dyn Fn(&mut Window, &mut App) -> AnyElement>;

pub enum IconSource {
    Kind(IconKind),
    Custom(ElementBuilder),
}

impl IconSource {
    pub fn element(&self, size: f32, window: &mut Window, cx: &mut App) -> AnyElement {
        match self {
            Self::Kind(kind) => Icon::new(*kind).size(size).into_any_element(),
            Self::Custom(builder) => builder(window, cx),
        }
    }
}

pub trait IntoIconSource {
    fn into_icon_source(self) -> IconSource;
}

impl IntoIconSource for IconKind {
    fn into_icon_source(self) -> IconSource {
        IconSource::Kind(self)
    }
}

impl<F> IntoIconSource for F
where
    F: Fn(&mut Window, &mut App) -> AnyElement + 'static,
{
    fn into_icon_source(self) -> IconSource {
        IconSource::Custom(Box::new(self))
    }
}
pub type ChangeHandler<T> = Rc<dyn Fn(T, &mut Window, &mut App)>;

pub fn sync_enabled(controller: &Entity<InteractionController>, enabled: bool, cx: &mut App) {
    let disabled = !enabled;
    if controller.read(cx).is_disabled() != disabled {
        controller.update(cx, |state, cx| state.set_disabled(disabled, cx));
    }
}
