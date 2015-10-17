use scribe::buffer::{line_range, LineRange};

/// Abstract representation of a fixed-height section of the screen.
/// Used to determine visible ranges of lines based on previous state,
/// explicit line focus, and common scrolling implementation behaviours.
pub struct ScrollableRegion {
    height: usize,
    line_offset: usize,
}

#[derive(PartialEq, Debug)]
pub enum Visibility {
    AboveRegion,
    Visible(usize),
    BelowRegion,
}

impl ScrollableRegion {
    // The height of the scrollable region.
    pub fn height(&self) -> usize {
        self.height
    }
    
    // Determines the visible lines based on the current line offset and height.
    pub fn visible_range(&self) -> LineRange {
        line_range::new(self.line_offset, self.height + self.line_offset)
    }

    /// If necessary, moves the line offset such that the specified line is
    /// visible, using previous state to determine whether said line is at
    /// the top or bottom of the new visible range.
    pub fn scroll_into_view(&mut self, line: usize) {
        let range = self.visible_range();
        if line < range.start() {
            self.line_offset = line;
        } else if line >= range.end() {
            self.line_offset = line - self.height + 1;
        }
    }

    /// Converts an absolutely positioned line number into
    /// one relative to the scrollable regions visible range.
    /// The visibility type is based on whether or not the line
    /// is outside of the region's visible range.
    pub fn relative_position(&self, line: usize) -> Visibility {
        match line.checked_sub(self.line_offset) {
            Some(line) => {
                if line >= self.height {
                    Visibility::BelowRegion
                } else {
                    Visibility::Visible(line)
                }
            },
            None => Visibility::AboveRegion,
        }
    }

    /// The number of lines the region has scrolled over.
    /// A value of zero represents an unscrolled region.
    pub fn line_offset(&self) -> usize {
        self.line_offset
    }

    pub fn scroll_up(&mut self, amount: usize) {
        self.line_offset = match self.line_offset.checked_sub(amount) {
            Some(amount) => amount,
            None => 0,
        };
    }

    pub fn scroll_down(&mut self, amount: usize) {
        self.line_offset += amount;
    }
}

pub fn new(height: usize) -> ScrollableRegion {
    ScrollableRegion{ height: height, line_offset: 0 }
}

#[cfg(test)]
mod tests {
    extern crate scribe;

    use super::{new, ScrollableRegion, Visibility};
    use scribe::buffer::line_range;

    #[test]
    fn visible_range_works_for_zero_based_line_offsets() {
        let height = 20;
        let region = new(height);
        let range = region.visible_range();
        assert_eq!(range.start(), 0);
        assert_eq!(range.end(), height);
    }

    #[test]
    fn visible_range_works_for_non_zero_line_offsets() {
        let region = ScrollableRegion{ height: 20, line_offset: 10 };
        let range = region.visible_range();
        assert_eq!(range.start(), 10);
        assert_eq!(range.end(), 30);
    }

    #[test]
    fn scroll_into_view_advances_region_if_line_after_current_range() {
        let mut region = ScrollableRegion{ height: 20, line_offset: 10 };
        region.scroll_into_view(40);
        let range = region.visible_range();
        assert_eq!(range.start(), 21);
        assert_eq!(range.end(), 41);
    }

    #[test]
    fn scroll_into_view_recedes_region_if_line_before_current_range() {
        let mut region = ScrollableRegion{ height: 20, line_offset: 10 };
        region.scroll_into_view(5);
        let range = region.visible_range();
        assert_eq!(range.start(), 5);
        assert_eq!(range.end(), 25);
    }

    #[test]
    fn relative_position_returns_correct_value_when_positive() {
        let mut region = new(20);
        region.scroll_into_view(30);
        assert_eq!(region.relative_position(15), Visibility::Visible(4));
    }

    #[test]
    fn relative_position_returns_above_region_when_negative() {
        let mut region = new(20);
        region.scroll_into_view(30);
        assert_eq!(region.relative_position(0), Visibility::AboveRegion);
    }

    #[test]
    fn relative_position_returns_below_region_when_beyond_visible_range() {
        let region = new(20);
        assert_eq!(region.relative_position(20), Visibility::BelowRegion);
    }

    #[test]
    fn scroll_down_increases_line_offset_by_amount() {
        let mut region = new(20);
        region.scroll_down(10);
        assert_eq!(
            region.visible_range(),
            line_range::new(10, 30)
        );
    }

    #[test]
    fn scroll_up_decreases_line_offset_by_amount() {
        let mut region = new(20);
        region.scroll_down(10);
        region.scroll_up(5);
        assert_eq!(
            region.visible_range(),
            line_range::new(5, 25)
        );
    }

    #[test]
    fn scroll_up_does_not_scroll_beyond_top_of_region() {
        let mut region = new(20);
        region.scroll_up(5);
        assert_eq!(
            region.visible_range(),
            line_range::new(0, 20)
        );
    }
}
