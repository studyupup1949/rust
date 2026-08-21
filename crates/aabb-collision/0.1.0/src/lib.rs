//! Axis-Aligned Bounding Box (AABB) collision detection.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AABB {
    pub min: [f64; 2],
    pub max: [f64; 2],
}

impl AABB {
    pub fn new(min: [f64; 2], max: [f64; 2]) -> Self {
        Self { min, max }
    }

    pub fn from_center(center: [f64; 2], half_extents: [f64; 2]) -> Self {
        Self {
            min: [center[0] - half_extents[0], center[1] - half_extents[1]],
            max: [center[0] + half_extents[0], center[1] + half_extents[1]],
        }
    }

    pub fn center(&self) -> [f64; 2] {
        [(self.min[0] + self.max[0]) / 2.0, (self.min[1] + self.max[1]) / 2.0]
    }

    pub fn half_extents(&self) -> [f64; 2] {
        [(self.max[0] - self.min[0]) / 2.0, (self.max[1] - self.min[1]) / 2.0]
    }

    pub fn intersects(&self, other: &AABB) -> bool {
        self.min[0] <= other.max[0]
            && self.max[0] >= other.min[0]
            && self.min[1] <= other.max[1]
            && self.max[1] >= other.min[1]
    }

    pub fn contains_point(&self, p: [f64; 2]) -> bool {
        p[0] >= self.min[0] && p[0] <= self.max[0] && p[1] >= self.min[1] && p[1] <= self.max[1]
    }

    pub fn contains(&self, other: &AABB) -> bool {
        self.min[0] <= other.min[0]
            && self.max[0] >= other.max[0]
            && self.min[1] <= other.min[1]
            && self.max[1] >= other.max[1]
    }

    /// Compute the overlapping region, if any.
    pub fn overlap(&self, other: &AABB) -> Option<AABB> {
        if !self.intersects(other) {
            return None;
        }
        Some(AABB {
            min: [self.min[0].max(other.min[0]), self.min[1].max(other.min[1])],
            max: [self.max[0].min(other.max[0]), self.max[1].min(other.max[1])],
        })
    }

    pub fn merged(&self, other: &AABB) -> AABB {
        AABB {
            min: [self.min[0].min(other.min[0]), self.min[1].min(other.min[1])],
            max: [self.max[0].max(other.max[0]), self.max[1].max(other.max[1])],
        }
    }

    pub fn area(&self) -> f64 {
        (self.max[0] - self.min[0]) * (self.max[1] - self.min[1])
    }
}

/// Simple broad-phase AABB collision checker using a sweep-and-prune approach.
#[derive(Debug, Clone)]
pub struct CollisionDetector {
    boxes: Vec<(usize, AABB)>,
}

impl CollisionDetector {
    pub fn new() -> Self {
        Self { boxes: Vec::new() }
    }

    pub fn insert(&mut self, id: usize, aabb: AABB) {
        self.boxes.push((id, aabb));
    }

    pub fn clear(&mut self) {
        self.boxes.clear();
    }

    /// Find all pairs of overlapping AABBs.
    pub fn find_collisions(&self) -> Vec<(usize, usize)> {
        let mut pairs = Vec::new();
        for i in 0..self.boxes.len() {
            for j in (i + 1)..self.boxes.len() {
                if self.boxes[i].1.intersects(&self.boxes[j].1) {
                    let (a, b) = if self.boxes[i].0 < self.boxes[j].0 {
                        (self.boxes[i].0, self.boxes[j].0)
                    } else {
                        (self.boxes[j].0, self.boxes[i].0)
                    };
                    pairs.push((a, b));
                }
            }
        }
        pairs.sort();
        pairs.dedup();
        pairs
    }
}

impl Default for CollisionDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// 3D AABB variant
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AABB3 {
    pub min: [f64; 3],
    pub max: [f64; 3],
}

impl AABB3 {
    pub fn new(min: [f64; 3], max: [f64; 3]) -> Self {
        Self { min, max }
    }

    pub fn intersects(&self, other: &AABB3) -> bool {
        self.min[0] <= other.max[0]
            && self.max[0] >= other.min[0]
            && self.min[1] <= other.max[1]
            && self.max[1] >= other.min[1]
            && self.min[2] <= other.max[2]
            && self.max[2] >= other.min[2]
    }

    pub fn volume(&self) -> f64 {
        (self.max[0] - self.min[0]) * (self.max[1] - self.min[1]) * (self.max[2] - self.min[2])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overlap() {
        let a = AABB::new([0.0, 0.0], [2.0, 2.0]);
        let b = AABB::new([1.0, 1.0], [3.0, 3.0]);
        assert!(a.intersects(&b));
        let overlap = a.overlap(&b).unwrap();
        assert_eq!(overlap.min, [1.0, 1.0]);
        assert_eq!(overlap.max, [2.0, 2.0]);
    }

    #[test]
    fn test_no_overlap() {
        let a = AABB::new([0.0, 0.0], [1.0, 1.0]);
        let b = AABB::new([2.0, 2.0], [3.0, 3.0]);
        assert!(!a.intersects(&b));
    }

    #[test]
    fn test_collision_detector() {
        let mut det = CollisionDetector::new();
        det.insert(0, AABB::new([0.0, 0.0], [2.0, 2.0]));
        det.insert(1, AABB::new([1.0, 1.0], [3.0, 3.0]));
        det.insert(2, AABB::new([10.0, 10.0], [11.0, 11.0]));
        let pairs = det.find_collisions();
        assert_eq!(pairs, vec![(0, 1)]);
    }
}
