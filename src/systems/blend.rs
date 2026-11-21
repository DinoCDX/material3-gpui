use gpui::{Hsla, Rgba};

pub fn with_alpha(color: Hsla, alpha: f32) -> Hsla {
    Hsla { a: alpha, ..color }
}

pub fn composite_over(top: Hsla, bottom: Hsla) -> Hsla {
    let top = Rgba::from(top);
    let bottom = Rgba::from(bottom);
    let alpha = top.a + bottom.a * (1.0 - top.a);
    if alpha <= 0.0 {
        return Hsla::from(Rgba {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        });
    }
    let mix = |t: f32, b: f32| (t * top.a + b * bottom.a * (1.0 - top.a)) / alpha;
    Hsla::from(Rgba {
        r: mix(top.r, bottom.r),
        g: mix(top.g, bottom.g),
        b: mix(top.b, bottom.b),
        a: alpha,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{black, white};

    fn assert_rgb(actual: Hsla, expected: f32) {
        let rgba = Rgba::from(actual);
        assert!((rgba.r - expected).abs() < 1e-3);
        assert!((rgba.g - expected).abs() < 1e-3);
        assert!((rgba.b - expected).abs() < 1e-3);
    }

    #[test]
    fn half_transparent_black_over_white_is_mid_gray() {
        assert_rgb(composite_over(with_alpha(black(), 0.5), white()), 0.5);
    }

    #[test]
    fn fully_transparent_top_keeps_bottom() {
        assert_rgb(composite_over(with_alpha(black(), 0.0), white()), 1.0);
    }

    #[test]
    fn opaque_top_hides_bottom() {
        assert_rgb(composite_over(black(), white()), 0.0);
    }
}
