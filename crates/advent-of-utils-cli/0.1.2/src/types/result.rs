use std::{
    collections::{BTreeSet, HashMap},
    time::Duration,
};

use super::{
    display::{Table, TableStruct},
    time::AocDuration,
    Parts,
};
use advent_of_utils::AocOption;

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct AocResult {
    day: u8,
    part: Parts,
    result: AocOption,
    time: AocDuration,
}

impl AocResult {
    pub fn new(day: u8, part: u8, result: AocOption, time: Vec<Duration>) -> Self {
        Self {
            day,
            part: Parts::new(part).unwrap(),
            result,
            time: AocDuration::new(time),
        }
    }

    pub fn day(&self) -> u8 {
        self.day
    }
    pub fn part(&self) -> Parts {
        self.part
    }
    pub fn result(&self) -> &AocOption {
        &self.result
    }
}

pub struct AocYear {
    days: HashMap<(u8, Parts), AocResult>,
    available_days: BTreeSet<u8>,
}

impl AocYear {
    pub fn from_vec(results: Vec<AocResult>) -> Self {
        let mut days: HashMap<(u8, Parts), AocResult> = HashMap::new();
        let mut available_days = BTreeSet::new();
        for mut result in results {
            let day = result.day();
            let part = result.part();
            let key = (day, part);

            match days.get_mut(&key) {
                Some(r) => {
                    r.time.additional_time(result.time.get_mut_time());
                }
                None => {
                    available_days.insert(day);
                    days.insert(key, result);
                }
            }
        }
        Self {
            days,
            available_days,
        }
    }
    fn calculate_total_time(&self) -> AocDuration {
        let mut total_durations = vec![];

        // For each run, sum up all times
        let max_runs = self
            .days
            .values()
            .map(|r| r.time.duration_len())
            .max()
            .unwrap_or(0);

        for i in 0..max_runs {
            let mut total = Duration::from_secs(0);
            for result in self.days.values() {
                if let Some(duration) = result.time.duration().get(i) {
                    total += *duration;
                }
            }
            total_durations.push(total);
        }

        AocDuration::new(total_durations)
    }
}

impl Table for AocYear {
    fn table_constructor(&self) -> TableStruct {
        let mut contents: Vec<Vec<String>> = vec![vec![
            "Day".to_string(),
            "Part 1".to_string(),
            "Part 2".to_string(),
            "Avg. Time 1".to_string(),
            "Avg. Time 2".to_string(),
        ]];
        let results = self.days.clone();
        for day in self.available_days.iter() {
            let part1 = (*day, Parts::Part1);
            let part2 = (*day, Parts::Part2);
            contents.push(vec![
                day.to_string(),
                results
                    .get(&part1)
                    .map(|r| r.result.clone())
                    .unwrap_or(AocOption::None)
                    .to_string(),
                results
                    .get(&part2)
                    .map(|r| r.result.clone())
                    .unwrap_or(AocOption::None)
                    .to_string(),
                results
                    .get(&part1)
                    .map(|r| r.time.clone())
                    .unwrap_or_default()
                    .to_string(),
                results
                    .get(&part2)
                    .map(|r| r.time.clone())
                    .unwrap_or_default()
                    .to_string(),
            ])
        }
        TableStruct::new(contents)
    }

    fn reduced_table_constructor(&self) -> TableStruct {
        let mut contents: Vec<Vec<String>> = vec![vec![
            "Day".to_string(),
            "Part 1".to_string(),
            "Part 2".to_string(),
        ]];
        let results = self.days.clone();
        for day in self.available_days.iter() {
            let part1 = (*day, Parts::Part1);
            let part2 = (*day, Parts::Part2);
            contents.push(vec![
                day.to_string(),
                results
                    .get(&part1)
                    .map(|r| r.result.clone())
                    .unwrap_or(AocOption::None)
                    .to_string(),
                results
                    .get(&part2)
                    .map(|r| r.result.clone())
                    .unwrap_or(AocOption::None)
                    .to_string(),
            ])
        }
        TableStruct::new(contents)
    }
}
