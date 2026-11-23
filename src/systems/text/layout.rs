pub fn glyph_positions(advances: &[f32], kerning: &[f32], tracking: f32) -> (Vec<f32>, f32) {
    let mut positions = Vec::with_capacity(advances.len());
    let mut x = 0.0_f32;
    for (index, advance) in advances.iter().enumerate() {
        positions.push(x);
        let kern = kerning.get(index).copied().unwrap_or(0.0);
        x += advance + kern + tracking;
    }
    let width = match (positions.last(), advances.last()) {
        (Some(last), Some(advance)) => last + advance,
        _ => 0.0,
    };
    (positions, width)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracking_and_kerning_move_each_glyph() {
        let (positions, width) = glyph_positions(&[10.0, 10.0, 10.0], &[-1.0, 0.0], 2.0);
        assert_eq!(positions, vec![0.0, 11.0, 23.0]);
        assert_eq!(width, 33.0);
    }

    #[test]
    fn zero_tracking_keeps_natural_advances() {
        let (positions, width) = glyph_positions(&[4.0, 6.0], &[0.0], 0.0);
        assert_eq!(positions, vec![0.0, 4.0]);
        assert_eq!(width, 10.0);
    }

    #[test]
    fn empty_text_has_no_width() {
        let (positions, width) = glyph_positions(&[], &[], 3.0);
        assert!(positions.is_empty());
        assert_eq!(width, 0.0);
    }
}
