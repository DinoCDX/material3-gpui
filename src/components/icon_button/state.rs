pub fn next_selected(current: bool) -> bool {
    !current
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggling_flips_the_selected_state() {
        assert!(next_selected(false));
        assert!(!next_selected(true));
    }
}
