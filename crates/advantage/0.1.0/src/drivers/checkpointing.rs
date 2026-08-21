use super::*;
use num::bigint::BigUint;
use num::integer::binomial;
use num::ToPrimitive;
use std::collections::VecDeque;
use std::f64::consts::PI;
use std::fmt::Debug;
use std::iter::Iterator;

fn beta(c: isize, t: isize) -> usize {
    if c <= 0 || t <= 0 {
        0
    } else {
        let c = BigUint::from(c as usize);
        let t = BigUint::from(t as usize);
        binomial(&c + t, c).to_usize().unwrap()
    }
}

fn find_t(c: usize, r: usize) -> usize {
    let c = c as isize;
    let r = r as isize;

    if c == 1 {
        if r > 0 {
            (r - 1) as usize
        } else {
            0
        }
    } else {
        let mut t = {
            let c = c as f64;
            let r = r as f64;
            ((-1.0).exp() * c * ((2.0 * PI * c).sqrt() * r).powf(1.0 / c) - c).ceil() as isize
        };
        while r > beta(c, t) as isize {
            t += 1;
        }
        assert!((beta(c, t - 1) as isize) < r);
        assert!(r <= (beta(c, t) as isize));
        t as usize
    }
}

fn next_checkpoint(c: usize, r: usize) -> usize {
    let t = find_t(c, r) as isize;
    let c = c as isize;

    if c <= 0 || r == 0 {
        0
    } else if r <= (beta(c, t - 1) + beta(c - 2, t - 1)) {
        beta(c, t - 2)
    } else if r >= (beta(c, t) - beta(c - 3, t)) {
        beta(c, t - 1)
    } else {
        r - beta(c - 1, t - 1) - beta(c - 2, t - 1)
    }
}

fn schedule(c: usize, r: usize) -> impl Iterator<Item = usize> {
    let mut offset = 0;
    let mut c_ = c;
    let mut r_ = r;
    std::iter::from_fn(move || {
        if r_ > 1 && c_ > 0 {
            let partial_cp = if (r_ - 1) <= c_ {
                1
            } else if c_ == 1 {
                r_ - 1
            } else if r_ > 2 {
                next_checkpoint(c_, r_).max(1).min(r_ - 1)
            } else {
                1
            };
            let cp = offset + partial_cp;
            offset = cp;
            r_ = r - cp;
            c_ -= 1;
            Some(cp)
        } else {
            None
        }
    })
}

/// Generate a sequence and walk it in reverse using limited memory
pub fn reverse_sequence<T, FW, RV, R, ID>(
    x: T,
    nsteps: usize,
    ncheckpoints: usize,
    forward: FW,
    reverse: RV,
    identity: ID,
) -> R
where
    T: Clone + Debug,
    R: Debug,
    FW: Fn(T) -> T,
    RV: Fn(T, R) -> R,
    ID: Fn(T) -> R,
{
    // We need at least 2 checkpoints: One for the beginning of the sequence and one for the end.
    assert!(ncheckpoints >= 2);

    // Store x at index 0 as the first checkpoint
    let mut checkpoints: VecDeque<(usize, T)> = VecDeque::with_capacity(ncheckpoints);
    checkpoints.push_back((0, x));

    // Length of the not-yet reversed sequence
    let mut r = nsteps + 1;

    let mut result: Option<R> = None;
    while r > 0 {
        // Forward until the end of the sequence while recording checkpoints
        {
            // Index of the last checkpoint that was recorded
            let (last_cp_idx, last_cp) = checkpoints.back().unwrap().clone();
            // Length of the sequence that we need a checkpoint schedule for
            let partial_r = r - last_cp_idx;
            // Number of checkpoints that we have left
            let partial_c = ncheckpoints - checkpoints.len();

            // Iterate through the checkpointing schedule for the partial sequence
            let mut current_i = 0;
            let mut current_x = last_cp;
            if partial_r > 0 {
                for cp_idx in schedule(partial_c, partial_r) {
                    for _ in current_i..cp_idx {
                        current_x = forward(current_x);
                    }
                    current_i = cp_idx;
                    checkpoints.push_back((last_cp_idx + current_i, current_x.clone()));
                }
            }
        }

        // Reverse element at the end of the checkpoints array
        {
            let (cp_idx, cp) = checkpoints.pop_back().unwrap();
            assert_eq!(cp_idx, r - 1);
            result = match result {
                Some(right) => Some(reverse(cp, right)),
                None => Some(identity(cp)),
            };
            r -= 1;
        }
    }

    result.unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_t_tests() {
        assert_eq!(find_t(3, 100), 7);
    }

    #[test]
    fn schedule_tests() {
        assert_eq!(&schedule(0, 100).collect::<Vec<usize>>(), &[]);
        assert_eq!(&schedule(1, 100).collect::<Vec<usize>>(), &[99]);
        assert_eq!(&schedule(2, 100).collect::<Vec<usize>>(), &[87, 99]);
        assert_eq!(&schedule(3, 100).collect::<Vec<usize>>(), &[65, 93, 99]);
        assert_eq!(&schedule(4, 100).collect::<Vec<usize>>(), &[50, 80, 95, 99]);
        assert_eq!(
            &schedule(5, 100).collect::<Vec<usize>>(),
            &[45, 70, 86, 96, 99]
        );

        assert_eq!(&schedule(42, 0).collect::<Vec<usize>>(), &[]);
        assert_eq!(&schedule(42, 1).collect::<Vec<usize>>(), &[]);
        assert_eq!(&schedule(42, 2).collect::<Vec<usize>>(), &[1]);
        assert_eq!(&schedule(42, 3).collect::<Vec<usize>>(), &[1, 2]);
        assert_eq!(&schedule(42, 4).collect::<Vec<usize>>(), &[1, 2, 3]);

        assert_eq!(
            &schedule(42, 41).collect::<Vec<usize>>(),
            &(1..41).collect::<Vec<usize>>()
        );

        assert_eq!(schedule(5, 5).collect::<Vec<usize>>(), &[1, 2, 3, 4]);
    }

    #[test]
    fn schedule_consistency() {
        let c = 4;
        let r = 100;
        let full_schedule = schedule(c, r).collect::<Vec<usize>>();
        let first_cp = full_schedule.first().unwrap();
        let partial_schedule = schedule(c - 1, r - first_cp)
            .map(|cp| cp + first_cp)
            .collect::<Vec<usize>>();
        assert_eq!(&partial_schedule[..], &full_schedule[1..]);
    }

    #[test]
    fn sequence_reverse_gauss_sum() {
        let r = 37;
        let reference = r * (r + 1) / 2;
        let result = reverse_sequence(0, r, 9, |x| x + 1, |x, y| x + y, |x| x);
        assert_eq!(result, reference);
    }
}
