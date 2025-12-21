use crate::TextSize;

/// A range in source text, represented as a half-open interval `[start, end)`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct TextRange {
    start: TextSize,
    end: TextSize,
}

impl TextRange {
    /// Creates a new `TextRange` with the given start and end.
    ///
    /// # Panics
    /// Panics if `start > end`.
    pub fn new(start: TextSize, end: TextSize) -> Self {
        assert!(start <= end, "start must be <= end");
        Self { start, end }
    }

    /// Returns the start of this range.
    pub const fn start(self) -> TextSize {
        self.start
    }

    /// Returns the end of this range.
    pub const fn end(self) -> TextSize {
        self.end
    }

    /// Returns the length of this range.
    pub fn len(self) -> TextSize {
        self.end - self.start
    }

    /// Returns `true` if this range is empty.
    pub fn is_empty(self) -> bool {
        self.start == self.end
    }

    /// Returns `true` if this range contains the given offset.
    ///
    /// The end is exclusive: `[start, end)`
    pub fn contains(self, offset: TextSize) -> bool {
        self.start <= offset && offset < self.end
    }

    /// Returns `true` if this range contains the given offset (inclusive end).
    ///
    /// The end is inclusive: `[start, end]`
    pub fn contains_inclusive(self, offset: TextSize) -> bool {
        self.start <= offset && offset <= self.end
    }

    /// Returns `true` if this range fully contains another range.
    pub fn contains_range(self, other: Self) -> bool {
        self.start <= other.start && other.end <= self.end
    }

    /// Returns the intersection of two ranges, or `None` if they don't overlap.
    pub fn intersect(self, other: Self) -> Option<Self> {
        let start = self.start.max(other.start);
        let end = self.end.min(other.end);
        if start <= end {
            Some(Self { start, end })
        } else {
            None
        }
    }

    /// Returns the smallest range that covers both ranges.
    pub fn cover(self, other: Self) -> Self {
        let start = self.start.min(other.start);
        let end = self.end.max(other.end);
        Self { start, end }
    }

    /// Returns the smallest range that covers this range and the given offset.
    pub fn cover_offset(self, offset: TextSize) -> Self {
        if offset < self.start {
            Self {
                start: offset,
                end: self.end,
            }
        } else if offset > self.end {
            Self {
                start: self.start,
                end: offset,
            }
        } else {
            self
        }
    }
}
