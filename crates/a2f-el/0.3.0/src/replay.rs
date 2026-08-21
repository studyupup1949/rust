// src/replay.rs
//! リプレイ攻撃対策（スライディングウィンドウ）

use std::collections::VecDeque;

/// スライディングウィンドウ（リプレイ対策＋順序自由）
pub struct SlidingWindow {
    window_size: u64,
    min_seq: u64,
    received: VecDeque<bool>,
}

impl SlidingWindow {
    pub fn new(window_size: u64) -> Self {
        Self {
            window_size,
            min_seq: 0,
            received: VecDeque::from(vec![false; window_size as usize]),
        }
    }

    /// シーケンス番号をチェックし、新規なら記録する
    /// returns: true = 新規（処理OK）, false = リプレイ（弾く）
    pub fn check_and_record(&mut self, seq: u64) -> bool {
        if seq < self.min_seq {
            return false;
        }

        if seq >= self.min_seq + self.window_size {
            let shift = seq - (self.min_seq + self.window_size) + 1;
            for _ in 0..shift {
                self.received.pop_front();
                self.received.push_back(false);
            }
            self.min_seq += shift;
        }

        let index = (seq - self.min_seq) as usize;
        if self.received[index] {
            return false;
        }

        self.received[index] = true;
        true
    }

    #[allow(dead_code)]
    pub fn get_min_seq(&self) -> u64 {
        self.min_seq
    }
}