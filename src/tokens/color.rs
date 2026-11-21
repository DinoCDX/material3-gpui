#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ColorRole {
    Primary,
    OnPrimary,
    PrimaryContainer,
    OnPrimaryContainer,
    InversePrimary,
    Secondary,
    OnSecondary,
    SecondaryContainer,
    OnSecondaryContainer,
    Tertiary,
    OnTertiary,
    TertiaryContainer,
    OnTertiaryContainer,
    Error,
    OnError,
    ErrorContainer,
    OnErrorContainer,
    Background,
    OnBackground,
    Surface,
    SurfaceDim,
    SurfaceBright,
    SurfaceTint,
    SurfaceContainerLowest,
    SurfaceContainerLow,
    SurfaceContainer,
    SurfaceContainerHigh,
    SurfaceContainerHighest,
    OnSurface,
    SurfaceVariant,
    OnSurfaceVariant,
    Outline,
    OutlineVariant,
    InverseSurface,
    InverseOnSurface,
    Shadow,
    Scrim,
}

impl ColorRole {
    pub const ALL: [ColorRole; 37] = [
        ColorRole::Primary,
        ColorRole::OnPrimary,
        ColorRole::PrimaryContainer,
        ColorRole::OnPrimaryContainer,
        ColorRole::InversePrimary,
        ColorRole::Secondary,
        ColorRole::OnSecondary,
        ColorRole::SecondaryContainer,
        ColorRole::OnSecondaryContainer,
        ColorRole::Tertiary,
        ColorRole::OnTertiary,
        ColorRole::TertiaryContainer,
        ColorRole::OnTertiaryContainer,
        ColorRole::Error,
        ColorRole::OnError,
        ColorRole::ErrorContainer,
        ColorRole::OnErrorContainer,
        ColorRole::Background,
        ColorRole::OnBackground,
        ColorRole::Surface,
        ColorRole::SurfaceDim,
        ColorRole::SurfaceBright,
        ColorRole::SurfaceTint,
        ColorRole::SurfaceContainerLowest,
        ColorRole::SurfaceContainerLow,
        ColorRole::SurfaceContainer,
        ColorRole::SurfaceContainerHigh,
        ColorRole::SurfaceContainerHighest,
        ColorRole::OnSurface,
        ColorRole::SurfaceVariant,
        ColorRole::OnSurfaceVariant,
        ColorRole::Outline,
        ColorRole::OutlineVariant,
        ColorRole::InverseSurface,
        ColorRole::InverseOnSurface,
        ColorRole::Shadow,
        ColorRole::Scrim,
    ];
    pub const COUNT: usize = Self::ALL.len();
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KeyRole {
    Primary,
    Secondary,
    Tertiary,
    Error,
    Neutral,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyRoleColors {
    pub color: ColorRole,
    pub on_color: ColorRole,
    pub container: ColorRole,
    pub on_container: ColorRole,
}

impl KeyRole {
    pub const fn colors(self) -> KeyRoleColors {
        match self {
            Self::Primary => KeyRoleColors {
                color: ColorRole::Primary,
                on_color: ColorRole::OnPrimary,
                container: ColorRole::PrimaryContainer,
                on_container: ColorRole::OnPrimaryContainer,
            },
            Self::Secondary => KeyRoleColors {
                color: ColorRole::Secondary,
                on_color: ColorRole::OnSecondary,
                container: ColorRole::SecondaryContainer,
                on_container: ColorRole::OnSecondaryContainer,
            },
            Self::Tertiary => KeyRoleColors {
                color: ColorRole::Tertiary,
                on_color: ColorRole::OnTertiary,
                container: ColorRole::TertiaryContainer,
                on_container: ColorRole::OnTertiaryContainer,
            },
            Self::Error => KeyRoleColors {
                color: ColorRole::Error,
                on_color: ColorRole::OnError,
                container: ColorRole::ErrorContainer,
                on_container: ColorRole::OnErrorContainer,
            },
            Self::Neutral => KeyRoleColors {
                color: ColorRole::Surface,
                on_color: ColorRole::OnSurface,
                container: ColorRole::SurfaceVariant,
                on_container: ColorRole::OnSurfaceVariant,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SurfaceTier {
    Lowest,
    Low,
    Container,
    High,
    Highest,
}

impl SurfaceTier {
    pub const fn role(self) -> ColorRole {
        match self {
            Self::Lowest => ColorRole::SurfaceContainerLowest,
            Self::Low => ColorRole::SurfaceContainerLow,
            Self::Container => ColorRole::SurfaceContainer,
            Self::High => ColorRole::SurfaceContainerHigh,
            Self::Highest => ColorRole::SurfaceContainerHighest,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_indices_match_declaration_order() {
        for (index, role) in ColorRole::ALL.iter().enumerate() {
            assert_eq!(*role as usize, index);
        }
    }

    #[test]
    fn neutral_key_role_maps_to_surface_family() {
        let colors = KeyRole::Neutral.colors();
        assert_eq!(colors.color, ColorRole::Surface);
        assert_eq!(colors.on_color, ColorRole::OnSurface);
    }

    #[test]
    fn surface_tiers_cover_lowest_through_highest() {
        assert_eq!(
            SurfaceTier::Lowest.role(),
            ColorRole::SurfaceContainerLowest
        );
        assert_eq!(
            SurfaceTier::Highest.role(),
            ColorRole::SurfaceContainerHighest
        );
    }
}
