pub const TRACK_GAP: f32 = 6.0;
pub const INSIDE_CORNER: f32 = 2.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Piece {
    pub left: f32,
    pub right: f32,
    pub round_left: f32,
    pub round_right: f32,
    pub active: bool,
}

pub fn single_pieces(width: f32, thumb_centre: f32, thumb_width: f32, full: f32) -> Vec<Piece> {
    let thumb_left = thumb_centre - thumb_width / 2.0;
    let thumb_right = thumb_centre + thumb_width / 2.0;
    let active_end = (thumb_left - TRACK_GAP).clamp(0.0, width);
    let inactive_start = (thumb_right + TRACK_GAP).clamp(0.0, width);
    let mut pieces = Vec::with_capacity(2);
    if active_end > 0.0 {
        pieces.push(Piece {
            left: 0.0,
            right: active_end,
            round_left: full,
            round_right: INSIDE_CORNER,
            active: true,
        });
    }
    if inactive_start < width {
        pieces.push(Piece {
            left: inactive_start,
            right: width,
            round_left: INSIDE_CORNER,
            round_right: full,
            active: false,
        });
    }
    pieces
}

pub fn range_pieces(
    width: f32,
    low_centre: f32,
    high_centre: f32,
    thumb_width: f32,
    full: f32,
) -> Vec<Piece> {
    let low_left = low_centre - thumb_width / 2.0;
    let low_right = low_centre + thumb_width / 2.0;
    let high_left = high_centre - thumb_width / 2.0;
    let high_right = high_centre + thumb_width / 2.0;
    let mut pieces = Vec::with_capacity(3);
    let outer_left_end = (low_left - TRACK_GAP).clamp(0.0, width);
    if outer_left_end > 0.0 {
        pieces.push(Piece {
            left: 0.0,
            right: outer_left_end,
            round_left: full,
            round_right: INSIDE_CORNER,
            active: false,
        });
    }
    let active_start = (low_right + TRACK_GAP).clamp(0.0, width);
    let active_end = (high_left - TRACK_GAP).clamp(0.0, width);
    if active_end > active_start {
        pieces.push(Piece {
            left: active_start,
            right: active_end,
            round_left: INSIDE_CORNER,
            round_right: INSIDE_CORNER,
            active: true,
        });
    }
    let outer_right_start = (high_right + TRACK_GAP).clamp(0.0, width);
    if outer_right_start < width {
        pieces.push(Piece {
            left: outer_right_start,
            right: width,
            round_left: INSIDE_CORNER,
            round_right: full,
            active: false,
        });
    }
    pieces
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_slider_at_the_start_has_no_active_piece() {
        let pieces = single_pieces(240.0, 0.0, 4.0, 8.0);
        assert_eq!(pieces.len(), 1);
        assert!(!pieces[0].active);
        assert_eq!(pieces[0].left, 2.0 + TRACK_GAP);
    }

    #[test]
    fn single_slider_gaps_both_sides_of_the_thumb() {
        let pieces = single_pieces(240.0, 120.0, 4.0, 8.0);
        assert_eq!(pieces.len(), 2);
        assert_eq!(pieces[0].right, 118.0 - TRACK_GAP);
        assert_eq!(pieces[1].left, 122.0 + TRACK_GAP);
    }

    #[test]
    fn range_slider_produces_three_pieces_when_the_thumbs_are_apart() {
        let pieces = range_pieces(240.0, 60.0, 180.0, 4.0, 8.0);
        assert_eq!(pieces.len(), 3);
        assert!(pieces[1].active);
        assert!(!pieces[0].active && !pieces[2].active);
    }
}
