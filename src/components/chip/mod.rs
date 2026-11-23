pub mod tokens;

use std::rc::Rc;

use gpui::AppContext;
use gpui::prelude::*;
use gpui::{
    App, Context, ElementId, Entity, Hsla, IntoElement, Render, SharedString, Subscription, Window,
    div, px, transparent_black,
};

use super::sync_enabled;
use crate::systems::text::TrackedText;
use crate::systems::blend::with_alpha;
use crate::systems::interaction::config::RippleConfig;
use crate::systems::interaction::controller::InteractionController;
use crate::systems::surface::{ActivateFn, StateElevation, Surface, SurfaceColors, SurfaceStyle};
use crate::systems::theme::{ActiveTheme, MaterialTheme};
use crate::tokens::color::ColorRole;
use crate::tokens::elevation::ElevationLevel;
use crate::tokens::shape::ShapeToken;
use crate::tokens::typography::TypeStyle;
use tokens::{
    CONTAINER_HEIGHT, DISABLED_CONTAINER_OPACITY, DISABLED_ICON_OPACITY, DISABLED_LABEL_OPACITY,
    DISABLED_OUTLINE_OPACITY, ICON_LABEL_SPACE, ICON_SIZE, PADDING_WITH_ICON,
    PADDING_WITHOUT_ICON,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ChipKind {
    Assist,
    Filter,
    Input,
    Suggestion,
}

impl ChipKind {
    pub fn supports_selection(self) -> bool {
        matches!(self, Self::Filter | Self::Input)
    }

    pub fn supports_elevation(self) -> bool {
        matches!(self, Self::Assist | Self::Filter | Self::Suggestion)
    }
}

#[derive(Clone, Copy, Debug)]
struct ChipColors {
    surface: SurfaceColors,
    icon: Hsla,
    trailing_icon: Hsla,
    label_text: Hsla,
}

fn chip_colors(
    theme: &MaterialTheme,
    kind: ChipKind,
    selected: bool,
    elevated: bool,
    disabled: bool,
) -> ChipColors {
    let on_surface = theme.color(ColorRole::OnSurface);
    let on_surface_variant = theme.color(ColorRole::OnSurfaceVariant);
    let primary = theme.color(ColorRole::Primary);
    let outline_variant = theme.color(ColorRole::OutlineVariant);
    let surface_low = theme.color(ColorRole::SurfaceContainerLow);
    let secondary_container = theme.color(ColorRole::SecondaryContainer);
    let on_secondary_container = theme.color(ColorRole::OnSecondaryContainer);
    let transparent = transparent_black();

    let is_selected = selected && kind.supports_selection();
    let label_color = match (kind, is_selected) {
        (ChipKind::Assist, _) => on_surface,
        (ChipKind::Suggestion, _) => on_surface_variant,
        (_, true) => on_secondary_container,
        (_, false) => on_surface_variant,
    };
    let icon_color = match (kind, is_selected) {
        (ChipKind::Input, true) => primary,
        (ChipKind::Input, false) => on_surface_variant,
        (_, true) => on_secondary_container,
        (_, false) => primary,
    };
    let trailing_icon_color = if is_selected {
        on_secondary_container
    } else {
        on_surface_variant
    };

    let container = if is_selected {
        secondary_container
    } else if elevated && kind.supports_elevation() {
        surface_low
    } else {
        transparent
    };
    let outline = outline_variant;

    if disabled {
        let muted = with_alpha(on_surface, DISABLED_LABEL_OPACITY);
        let disabled_container = with_alpha(on_surface, DISABLED_CONTAINER_OPACITY);
        let disabled_outline = with_alpha(on_surface, DISABLED_OUTLINE_OPACITY);
        let filled = is_selected || (elevated && kind.supports_elevation());
        return ChipColors {
            surface: SurfaceColors {
                container: if filled { disabled_container } else { transparent },
                content: muted,
                disabled_container: transparent,
                disabled_content: muted,
                outline: disabled_outline,
            },
            icon: with_alpha(on_surface, DISABLED_ICON_OPACITY),
            trailing_icon: with_alpha(on_surface, DISABLED_ICON_OPACITY),
            label_text: muted,
        };
    }

    ChipColors {
        surface: SurfaceColors {
            container,
            content: label_color,
            disabled_container: transparent,
            disabled_content: label_color,
            outline,
        },
        icon: icon_color,
        trailing_icon: trailing_icon_color,
        label_text: label_color,
    }
}

fn chip_elevation(kind: ChipKind, elevated: bool) -> StateElevation {
    if elevated && kind.supports_elevation() {
        StateElevation {
            default: ElevationLevel::Level1,
            hovered: ElevationLevel::Level2,
            focused: ElevationLevel::Level1,
            pressed: ElevationLevel::Level1,
            disabled: ElevationLevel::Level0,
        }
    } else {
        StateElevation::constant(ElevationLevel::Level0)
    }
}

pub struct Chip {
    id: ElementId,
    kind: ChipKind,
    label: SharedString,
    selected: bool,
    elevated: bool,
    enabled: bool,
    leading: Option<SharedString>,
    trailing: Option<SharedString>,
    on_click: Option<ActivateFn>,
    controller: Entity<InteractionController>,
    _observation: Subscription,
}

impl Chip {
    pub fn new(
        id: impl Into<ElementId>,
        kind: ChipKind,
        label: impl Into<SharedString>,
        cx: &mut Context<Self>,
    ) -> Self {
        let controller =
            cx.new(|cx| InteractionController::new(RippleConfig::new(gpui::black(), true), cx));
        let observation = cx.observe(&controller, |_, _, cx| cx.notify());
        Self {
            id: id.into(),
            kind,
            label: label.into(),
            selected: false,
            elevated: false,
            enabled: true,
            leading: None,
            trailing: None,
            on_click: None,
            controller,
            _observation: observation,
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn elevated(mut self, elevated: bool) -> Self {
        self.elevated = elevated;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn leading_icon(mut self, glyph: impl Into<SharedString>) -> Self {
        self.leading = Some(glyph.into());
        self
    }

    pub fn trailing_icon(mut self, glyph: impl Into<SharedString>) -> Self {
        self.trailing = Some(glyph.into());
        self
    }

    pub fn on_click(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    pub fn is_selected(&self) -> bool {
        self.selected
    }

    pub fn controller(&self) -> &Entity<InteractionController> {
        &self.controller
    }
}

impl Render for Chip {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        sync_enabled(&self.controller, self.enabled, cx);
        let disabled = self.controller.read(cx).is_disabled();
        let colors = chip_colors(cx.theme(), self.kind, self.selected, self.elevated, disabled);
        let typography = cx.theme().typography();
        let style = TypeStyle::LabelLarge;
        let family = typography.font_family(style.font()).clone();

        let has_leading = self.leading.is_some();
        let has_trailing = self.trailing.is_some() && self.kind == ChipKind::Input;
        let padding_left = if has_leading { PADDING_WITH_ICON } else { PADDING_WITHOUT_ICON };
        let padding_right = if has_trailing { PADDING_WITH_ICON } else { PADDING_WITHOUT_ICON };

        let mut row = div()
            .h(px(CONTAINER_HEIGHT))
            .flex()
            .items_center()
            .gap(px(ICON_LABEL_SPACE))
            .pl(px(padding_left))
            .pr(px(padding_right));
        if let Some(glyph) = self.leading.clone() {
            row = row.child(
                div()
                    .size(px(ICON_SIZE))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(colors.icon)
                    .child(glyph),
            );
        }
        row = row.child(TrackedText::new(
            self.label.clone(),
            style,
            colors.label_text,
            family,
        ));
        if has_trailing {
            if let Some(glyph) = self.trailing.clone() {
                row = row.child(
                    div()
                        .size(px(ICON_SIZE))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(colors.trailing_icon)
                        .child(glyph),
                );
            }
        }

        let view = cx.entity();
        let kind = self.kind;
        let next = !self.selected;
        let on_click = self.on_click.clone();
        Surface::new(self.id.clone())
            .style(SurfaceStyle {
                shape: ShapeToken::CornerSmall,
                tonal_elevation: ElevationLevel::Level0,
                elevation: chip_elevation(kind, self.elevated),
                outlined: !self.selected && !(self.elevated && kind.supports_elevation()),
            })
            .colors(colors.surface)
            .interactive(self.controller.clone())
            .on_activate(move |window, cx| {
                if kind.supports_selection() {
                    view.update(cx, |this, cx| {
                        this.selected = next;
                        cx.notify();
                    });
                }
                if let Some(handler) = &on_click {
                    (handler.as_ref())(window, cx);
                }
            })
            .child(row)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::theme::ThemeMode;
    use material_color_utils::utils::color_utils::Argb;

    fn theme() -> MaterialTheme {
        MaterialTheme::from_seed(Argb(0xFF4285F4), ThemeMode::Light)
    }

    #[test]
    fn only_filter_and_input_can_be_selected() {
        assert!(ChipKind::Filter.supports_selection());
        assert!(ChipKind::Input.supports_selection());
        assert!(!ChipKind::Assist.supports_selection());
        assert!(!ChipKind::Suggestion.supports_selection());
    }

    #[test]
    fn selected_filter_uses_secondary_container() {
        let theme = theme();
        let colors = chip_colors(&theme, ChipKind::Filter, true, false, false);
        assert_eq!(
            colors.surface.container.h,
            theme.color(ColorRole::SecondaryContainer).h
        );
    }

    #[test]
    fn elevated_assist_uses_surface_container_low() {
        let theme = theme();
        let colors = chip_colors(&theme, ChipKind::Assist, false, true, false);
        assert_eq!(
            colors.surface.container.h,
            theme.color(ColorRole::SurfaceContainerLow).h
        );
    }

    #[test]
    fn input_chips_are_never_elevated() {
        assert_eq!(
            chip_elevation(ChipKind::Input, true),
            StateElevation::constant(ElevationLevel::Level0)
        );
    }
}
