//! Anchor-based cursors and selections for [`List`] and [`Text`].
//!
//! A cursor is a position in a document that **follows concurrent edits
//! correctly**: if other replicas insert text before the cursor, it
//! shifts; if they insert text after, it stays put. This is the same
//! anchor model used in Yjs's `RelativePosition` and Automerge's
//! cursor API.
//!
//! [`Cursor`] is `Copy` and small (a single optional `OpId` plus a side
//! flag). It is **stable across serialization**: persist a cursor with
//! the rest of your CRDT state and the position it resolves to is
//! identical after deserialization, even if other replicas have edited
//! the document in between.
//!
//! ## Quick start
//!
//! ```
//! use abyo_crdt::{Cursor, List};
//!
//! let mut a = List::<char>::new(1);
//! a.insert(0, 'a');
//! a.insert(1, 'b');
//! a.insert(2, 'c');
//!
//! // Place a cursor between 'b' and 'c'.
//! let cur = Cursor::at(&a, 2);
//!
//! // Now insert at position 0 — the cursor's logical position shifts to 3.
//! a.insert(0, 'X');
//! assert_eq!(cur.resolve(&a), 3);
//! assert_eq!(a.to_vec(), vec!['X', 'a', 'b', 'c']);
//! ```

use crate::id::OpId;
use crate::list::List;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Which side of the anchored character the cursor sits on.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CursorSide {
    /// Cursor is immediately to the **left** of the anchored character.
    /// Edits at the cursor position go INTO the cursor's "after" range.
    Before,
    /// Cursor is immediately to the **right** of the anchored character.
    /// Edits at the cursor position go INTO the cursor's "before" range.
    After,
}

/// A position in a [`List`] that follows concurrent edits.
///
/// Internally just a character `OpId` + a side bit, so cursors are 16
/// bytes and `Copy`. They survive serialization, replica restart, and
/// arbitrary concurrent edits — the position they resolve to remains
/// well-defined as long as the anchored character (or a phantom for it
/// when tombstoned) is in the document.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Cursor {
    /// Cursor at the very start of the document. Always resolves to 0;
    /// new inserts at position 0 push the cursor right.
    Start,
    /// Cursor at the very end. Always resolves to `list.len()`; new
    /// inserts at the end push the cursor right.
    End,
    /// Cursor anchored to a specific character.
    Anchored {
        /// `OpId` of the character the cursor is anchored to.
        char_id: OpId,
        /// Which side of that character.
        side: CursorSide,
    },
}

impl Cursor {
    /// Place a cursor at visible position `pos` in `list`.
    ///
    /// - `pos == 0` and `list.is_empty()` → [`Cursor::Start`].
    /// - `pos == list.len()` → [`Cursor::End`].
    /// - Otherwise → anchored `Before` the character at position `pos`.
    pub fn at<T: Clone>(list: &List<T>, pos: usize) -> Self {
        let len = list.len();
        if list.is_empty() && pos == 0 {
            return Cursor::Start;
        }
        if pos >= len {
            return Cursor::End;
        }
        let char_id = list.id_at(pos).expect("pos < len");
        Cursor::Anchored {
            char_id,
            side: CursorSide::Before,
        }
    }

    /// Place a cursor immediately AFTER visible position `pos`.
    /// Useful for "insertion point on the right of this character" UX.
    pub fn after<T: Clone>(list: &List<T>, pos: usize) -> Self {
        let len = list.len();
        if pos + 1 >= len {
            return Cursor::End;
        }
        let char_id = list.id_at(pos).expect("pos < len");
        Cursor::Anchored {
            char_id,
            side: CursorSide::After,
        }
    }

    /// Resolve this cursor to a current visible position.
    ///
    /// Returns a value in `0..=list.len()`. The position is well-defined
    /// even after concurrent inserts/deletes; deleting the anchored
    /// character resolves to the position the cursor would naturally
    /// "fall to" (the visible item just after the deleted anchor).
    pub fn resolve<T: Clone>(self, list: &List<T>) -> usize {
        match self {
            Cursor::Start => 0,
            Cursor::End => list.len(),
            Cursor::Anchored { char_id, side } => {
                // If the anchor is visible, look it up directly.
                if let Some(pos) = list.position_of(char_id) {
                    return match side {
                        CursorSide::Before => pos,
                        CursorSide::After => pos + 1,
                    };
                }
                // The anchored char is tombstoned (or unknown). Fall back
                // to the phantom position: how many visible items precede
                // the anchored char in document order.
                list.phantom_position_of(char_id).unwrap_or(0)
            }
        }
    }
}

/// A range selection — two cursors marking the endpoints of a
/// contiguous span.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Selection {
    /// Anchor cursor (where the selection started).
    pub anchor: Cursor,
    /// Active cursor (where the selection ends; the user's "head").
    pub head: Cursor,
}

impl Selection {
    /// Selection over visible positions `range`. The anchor goes at
    /// `range.start` (Before-stickiness) and the head at `range.end`
    /// (After-stickiness on the previous char).
    pub fn over<T: Clone>(list: &List<T>, range: std::ops::Range<usize>) -> Self {
        Self {
            anchor: Cursor::at(list, range.start),
            head: if range.end == 0 {
                Cursor::Start
            } else {
                Cursor::after(list, range.end - 1)
            },
        }
    }

    /// Resolve to a visible-position range.
    ///
    /// Returns `(start, end)` with `start <= end`. The selection direction
    /// (head before/after anchor) is preserved in the original cursors but
    /// the resolved range is always normalized.
    pub fn resolve<T: Clone>(self, list: &List<T>) -> std::ops::Range<usize> {
        let a = self.anchor.resolve(list);
        let h = self.head.resolve(list);
        if a <= h {
            a..h
        } else {
            h..a
        }
    }

    /// Is the selection empty (anchor == head)?
    pub fn is_empty<T: Clone>(self, list: &List<T>) -> bool {
        self.anchor.resolve(list) == self.head.resolve(list)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build(s: &str) -> List<char> {
        let mut list = List::<char>::new(1);
        for (i, c) in s.chars().enumerate() {
            list.insert(i, c);
        }
        list
    }

    #[test]
    fn cursor_at_start_and_end() {
        let mut list = List::<char>::new(1);
        assert_eq!(Cursor::at(&list, 0), Cursor::Start);
        list.insert(0, 'a');
        list.insert(1, 'b');
        assert_eq!(Cursor::at(&list, 2), Cursor::End);
    }

    #[test]
    fn cursor_follows_inserts_before() {
        let mut list = build("hello");
        let c = Cursor::at(&list, 3); // before 'l' (the second 'l')
        assert_eq!(c.resolve(&list), 3);
        list.insert(0, 'X');
        // Now "Xhello", cursor still anchors before that 'l' at index 4.
        assert_eq!(c.resolve(&list), 4);
        list.insert(0, 'Y');
        assert_eq!(c.resolve(&list), 5);
    }

    #[test]
    fn cursor_does_not_move_for_inserts_after() {
        let mut list = build("hello");
        let c = Cursor::at(&list, 2); // before 'l' (first one)
        list.insert(5, '!'); // append
        assert_eq!(c.resolve(&list), 2);
    }

    #[test]
    fn cursor_resolves_after_deletion_of_anchor() {
        let mut list = build("hello");
        let c = Cursor::at(&list, 2); // before first 'l'
        list.delete(2); // delete 'l'
                        // Now "helo". The anchor's phantom position is 2 (index of 'l' that
                        // was at original 2; next visible 'l' is at new index 2).
        assert_eq!(c.resolve(&list), 2);
    }

    #[test]
    fn cursor_after_specific_char() {
        let mut list = build("abcd");
        let c = Cursor::after(&list, 1); // after 'b'
        assert_eq!(c.resolve(&list), 2);
        // Insert at position 0 — cursor still after 'b', now at index 3.
        list.insert(0, 'X');
        assert_eq!(c.resolve(&list), 3);
        // Insert AT the cursor position (between 'b' and 'c'): cursor stays
        // anchored after 'b', so the new char goes AFTER the cursor.
        // Wait — Cursor::After uses AnchorSide::After which means "right of
        // 'b'", so a new insert at that position would go to the cursor's
        // right side, leaving the cursor immediately after 'b'.
        let pos = c.resolve(&list);
        list.insert(pos, 'M');
        assert_eq!(list.to_vec(), vec!['X', 'a', 'b', 'M', 'c', 'd']);
        // Cursor still at position 3 (right after 'b').
        assert_eq!(c.resolve(&list), 3);
    }

    #[test]
    fn selection_over_range() {
        let mut list = build("hello world");
        let sel = Selection::over(&list, 6..11); // "world"
        assert_eq!(sel.resolve(&list), 6..11);

        // Insert at the beginning — selection should track.
        list.insert(0, 'X');
        assert_eq!(sel.resolve(&list), 7..12);
    }

    #[test]
    fn selection_emptiness() {
        let list = build("abc");
        let sel = Selection::over(&list, 1..1);
        assert!(sel.is_empty(&list));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn cursor_round_trips_via_json() {
        let list = build("hello");
        let c = Cursor::at(&list, 3);
        let json = serde_json::to_string(&c).unwrap();
        let restored: Cursor = serde_json::from_str(&json).unwrap();
        assert_eq!(c, restored);
        assert_eq!(restored.resolve(&list), 3);
    }
}
