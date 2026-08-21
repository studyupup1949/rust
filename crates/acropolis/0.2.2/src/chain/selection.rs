use crate::chain::ChainTip;

pub const TIE_MARK_SIZE: usize = 64;
const RESTRICTED_TIE_MAX_SLOT_DISTANCE: u64 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipOrdering {
    Equal,
    LeftBetter,
    RightBetter,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TieBreakMode {
    Unknown,
    Open,
    Narrow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TieBreakConfig {
    pub mode: TieBreakMode,
    pub max_slot_distance: u64,
}

impl TieBreakConfig {
    pub fn open() -> Self {
        Self {
            mode: TieBreakMode::Open,
            max_slot_distance: 0,
        }
    }

    pub fn restricted() -> Self {
        Self {
            mode: TieBreakMode::Narrow,
            max_slot_distance: RESTRICTED_TIE_MAX_SLOT_DISTANCE,
        }
    }

    pub fn unknown() -> Self {
        Self {
            mode: TieBreakMode::Unknown,
            max_slot_distance: 0,
        }
    }

    pub fn known(self) -> bool {
        matches!(self.mode, TieBreakMode::Open | TieBreakMode::Narrow)
    }
}

impl Default for TieBreakConfig {
    fn default() -> Self {
        Self::unknown()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TipSelectionView {
    pub slot: u64,
    pub issuer: Vec<u8>,
    pub issue_no: u64,
    pub tie_mark: Vec<u8>,
    pub tie_break: TieBreakConfig,
    has_issuer_and_issue: bool,
}

impl TipSelectionView {
    pub fn from_tip(tip: ChainTip, tie_mark: &[u8], tie_break: TieBreakConfig) -> Self {
        Self {
            slot: tip.point.slot,
            issuer: Vec::new(),
            issue_no: 0,
            tie_mark: tie_mark.to_vec(),
            tie_break,
            has_issuer_and_issue: false,
        }
    }

    pub fn full(
        tip: ChainTip,
        issuer: &[u8],
        issue_no: u64,
        tie_mark: &[u8],
        tie_break: TieBreakConfig,
    ) -> Self {
        Self {
            slot: tip.point.slot,
            issuer: issuer.to_vec(),
            issue_no,
            tie_mark: tie_mark.to_vec(),
            tie_break,
            has_issuer_and_issue: true,
        }
    }

    fn has_issuer_issue_no(&self) -> bool {
        self.has_issuer_and_issue && !self.issuer.is_empty()
    }
}

pub fn compare_chain_tips(
    left_tip: ChainTip,
    right_tip: ChainTip,
    left_view: &TipSelectionView,
    right_view: &TipSelectionView,
) -> TipOrdering {
    if left_tip.bundle_number > right_tip.bundle_number {
        return TipOrdering::LeftBetter;
    }
    if right_tip.bundle_number > left_tip.bundle_number {
        return TipOrdering::RightBetter;
    }
    if left_tip.point == right_tip.point {
        return TipOrdering::Equal;
    }
    if prefer_candidate_tip(right_view, left_view) {
        return TipOrdering::LeftBetter;
    }
    if prefer_candidate_tip(left_view, right_view) {
        return TipOrdering::RightBetter;
    }
    TipOrdering::Equal
}

pub fn prefer_candidate_tip(ours: &TipSelectionView, candidate: &TipSelectionView) -> bool {
    if issue_no_armed(ours, candidate) {
        if candidate.issue_no > ours.issue_no {
            return true;
        }
        if candidate.issue_no < ours.issue_no {
            return false;
        }
    }
    if !mark_armed(ours, candidate) {
        return false;
    }
    compare_tie_marks(&candidate.tie_mark, &ours.tie_mark) == TipOrdering::LeftBetter
}

pub fn compare_tie_marks(left: &[u8], right: &[u8]) -> TipOrdering {
    if left.len() != TIE_MARK_SIZE || right.len() != TIE_MARK_SIZE {
        return TipOrdering::Equal;
    }
    match left.cmp(right) {
        std::cmp::Ordering::Less => TipOrdering::LeftBetter,
        std::cmp::Ordering::Greater => TipOrdering::RightBetter,
        std::cmp::Ordering::Equal => TipOrdering::Equal,
    }
}

fn issue_no_armed(ours: &TipSelectionView, candidate: &TipSelectionView) -> bool {
    ours.slot == candidate.slot
        && ours.has_issuer_issue_no()
        && candidate.has_issuer_issue_no()
        && ours.issuer == candidate.issuer
}

fn mark_armed(ours: &TipSelectionView, candidate: &TipSelectionView) -> bool {
    let Some(tie_break) = tie_break_config(ours, candidate) else {
        return false;
    };
    match tie_break.mode {
        TieBreakMode::Unknown => false,
        TieBreakMode::Open => true,
        TieBreakMode::Narrow => ours.slot.abs_diff(candidate.slot) <= tie_break.max_slot_distance,
    }
}

fn tie_break_config(
    ours: &TipSelectionView,
    candidate: &TipSelectionView,
) -> Option<TieBreakConfig> {
    if ours.tie_break.known() {
        Some(ours.tie_break)
    } else if candidate.tie_break.known() {
        Some(candidate.tie_break)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::{ChainPoint, ChainTip};

    fn tip(slot: u64, bundle_number: u64, hash_byte: u8) -> ChainTip {
        ChainTip::new(ChainPoint::new(slot, [hash_byte; 32]), bundle_number)
    }

    #[test]
    fn higher_bundle_number_wins() {
        let left = tip(10, 10, 1);
        let right = tip(11, 11, 2);
        let left_view =
            TipSelectionView::from_tip(left, &[0; TIE_MARK_SIZE], TieBreakConfig::open());
        let right_view =
            TipSelectionView::from_tip(right, &[0; TIE_MARK_SIZE], TieBreakConfig::open());
        assert_eq!(
            compare_chain_tips(left, right, &left_view, &right_view),
            TipOrdering::RightBetter
        );
    }

    #[test]
    fn invalid_mark_lengths_do_not_win() {
        assert_eq!(
            compare_tie_marks(&[0; 63], &[1; TIE_MARK_SIZE]),
            TipOrdering::Equal
        );
    }

    #[test]
    fn lower_mark_wins_when_armed() {
        assert_eq!(
            compare_tie_marks(&[1; TIE_MARK_SIZE], &[2; TIE_MARK_SIZE]),
            TipOrdering::LeftBetter
        );
    }

    #[test]
    fn narrow_tie_ignores_distant_slots() {
        let ours_tip = tip(10, 9, 1);
        let candidate_tip = tip(20, 9, 2);
        let ours =
            TipSelectionView::from_tip(ours_tip, &[9; TIE_MARK_SIZE], TieBreakConfig::restricted());
        let candidate = TipSelectionView::from_tip(
            candidate_tip,
            &[1; TIE_MARK_SIZE],
            TieBreakConfig::restricted(),
        );
        assert!(!prefer_candidate_tip(&ours, &candidate));
    }

    #[test]
    fn higher_issue_number_wins_for_same_issuer_and_slot() {
        let ours_tip = tip(10, 9, 1);
        let candidate_tip = tip(10, 9, 2);
        let ours = TipSelectionView::full(
            ours_tip,
            b"issuer",
            1,
            &[9; TIE_MARK_SIZE],
            TieBreakConfig::open(),
        );
        let candidate = TipSelectionView::full(
            candidate_tip,
            b"issuer",
            2,
            &[8; TIE_MARK_SIZE],
            TieBreakConfig::open(),
        );
        assert!(prefer_candidate_tip(&ours, &candidate));
    }
}
