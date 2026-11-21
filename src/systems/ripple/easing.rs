use crate::tokens::motion::FAST_OUT_SLOW_IN_CONTROL_POINTS;

const BEZIER_SEARCH_STEPS: usize = 32;

pub fn fast_out_slow_in() -> impl Fn(f32) -> f32 {
    let (x1, y1, x2, y2) = FAST_OUT_SLOW_IN_CONTROL_POINTS;
    cubic_bezier(x1, y1, x2, y2)
}

pub fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32) -> impl Fn(f32) -> f32 {
    move |t: f32| {
        if t <= 0.0 {
            return 0.0;
        }
        if t >= 1.0 {
            return 1.0;
        }
        let (mut low, mut high) = (0.0_f32, 1.0_f32);
        for _ in 0..BEZIER_SEARCH_STEPS {
            let mid = (low + high) / 2.0;
            if bezier_component(mid, x1, x2) < t {
                low = mid;
            } else {
                high = mid;
            }
        }
        bezier_component((low + high) / 2.0, y1, y2)
    }
}

fn bezier_component(s: f32, p1: f32, p2: f32) -> f32 {
    let inverse = 1.0 - s;
    3.0 * inverse * inverse * s * p1 + 3.0 * inverse * s * s * p2 + s * s * s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curve_is_pinned_at_both_ends() {
        let curve = fast_out_slow_in();
        assert_eq!(curve(0.0), 0.0);
        assert_eq!(curve(1.0), 1.0);
    }

    #[test]
    fn fast_out_slow_in_leads_linear_at_midpoint() {
        assert!(fast_out_slow_in()(0.5) > 0.5);
    }
}
