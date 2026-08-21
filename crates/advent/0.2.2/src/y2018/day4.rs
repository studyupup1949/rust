// Copyright 2018 by Aldrin J D'Souza.
// Licensed under the MIT License <https://opensource.org/licenses/MIT>
//! Repose Record ([Statement](https://adventofcode.com/2018/day/4)).
//! Input size parameters `n`: Number of record entries, `g`: Number of guards

use std::collections::BTreeMap;

use super::super::{lines, parse, TRUST};

/// SleepLog is a mapping from guard to the number of times slept in a given minute `O(g)` space
pub type SleepLog = BTreeMap<usize, Vec<u32>>;

/// `O(n)` Read the sleep log from the given input
pub fn read(input: &str) -> SleepLog {
    // Read the logs and sort them
    let mut records: Vec<&str> = lines(input).collect();
    records.sort();

    // Initialize a new sleep log
    let mut log = SleepLog::new();

    // Initialize the indices
    let mut guard = 0;
    let mut since = 0;

    // Read the log and update the minutes slept
    for line in records {
        let numbers: Vec<usize> = parse(line, "[-:] # ").collect();

        if line.contains("Guard") {
            guard = numbers[5];
        }

        if line.contains("falls asleep") {
            since = numbers[4];
        }

        if line.contains("wakes up") {
            let now = numbers[4];
            let slept = log.entry(guard).or_insert_with(|| vec![0; 60]);

            for i in since..now {
                slept[i] += 1
            }
        }
    }

    // Done
    log
}

/// `O(g)` Pick a guard based on the given strategy
pub fn choose<F>(log: &SleepLog, strategy: F) -> (usize, usize)
where
    F: Fn(&[u32]) -> u32,
{
    let (sleepiest_guard, _) = log
        .iter()
        .max_by_key(|(_, sleep)| strategy(sleep))
        .expect(TRUST);

    let (sleepiest_minute, _) = log
        .get(sleepiest_guard)
        .expect(TRUST)
        .iter()
        .enumerate()
        .max_by_key(|(_, slept)| *slept)
        .expect(TRUST);

    (*sleepiest_guard, sleepiest_minute)
}

/// `O(g)` Find the sleepiest minute of the sleepiest guard
pub fn part1(log: &SleepLog) -> (usize, usize) {
    choose(log, |sleep| sleep.iter().sum())
}

/// `O(g)` Find the guard who is found sleeping most at a particular minute
pub fn part2(log: &SleepLog) -> (usize, usize) {
    choose(log, |sleep| *sleep.iter().max().expect(TRUST))
}

#[test]
fn examples() {
    let input = r"
[1518-11-01 00:00] Guard #10 begins shift
[1518-11-01 00:05] falls asleep
[1518-11-01 00:25] wakes up
[1518-11-01 00:30] falls asleep
[1518-11-01 00:55] wakes up
[1518-11-03 00:05] Guard #10 begins shift
[1518-11-01 23:58] Guard #99 begins shift
[1518-11-02 00:40] falls asleep
[1518-11-02 00:50] wakes up
[1518-11-03 00:24] falls asleep
[1518-11-04 00:36] falls asleep
[1518-11-03 00:29] wakes up
[1518-11-04 00:02] Guard #99 begins shift
[1518-11-05 00:03] Guard #99 begins shift
[1518-11-04 00:46] wakes up
[1518-11-05 00:45] falls asleep
[1518-11-05 00:55] wakes up
";
    let log = read(input);
    let (guard, minute) = part1(&log);

    assert_eq!(240, guard * minute);

    let (guard, minute) = part2(&log);
    assert_eq!(4455, guard * minute);
}

#[test]
fn solution() {
    let log = read(include_str!("input/4"));
    let (guard, minute) = part1(&log);
    assert_eq!(60438, guard * minute);

    let (guard, minute) = part2(&log);
    assert_eq!(47989, guard * minute);
}
