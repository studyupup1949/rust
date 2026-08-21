use std::{
    collections::{BTreeSet, HashMap},
    time::Duration,
};

use super::{
    display::{Table, TableStruct},
    Parts,
};
use advent_of_utils::AocOption;

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct AocResult {
    day: u8,
    part: Parts,
    result: AocOption,
    time: Vec<Duration>,
}

impl AocResult {
    pub fn new(day: u8, part: u8, result: AocOption, time: Duration) -> Self {
        Self {
            day,
            part: Parts::new(part).unwrap(),
            result,
            time: vec![time],
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
    pub fn get_mut_time(&mut self) -> &mut Vec<Duration> {
        &mut self.time
    }
    pub fn avg_time(&self) -> Duration {
        self.time.iter().sum::<Duration>() / self.time.len() as u32
    }
    pub fn additional_time(&mut self, time: &mut Vec<Duration>) {
        self.time.append(time);
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
                    r.additional_time(result.get_mut_time());
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
}

impl Table for AocYear {
    fn table_constructor(&self) -> TableStruct {
        let mut contents: Vec<Vec<String>> = vec![vec![
            "Day".to_string(),
            "Part 1".to_string(),
            "Part 2".to_string(),
            "Avg. Time 1 (μs)".to_string(),
            "Avg. Time 2 (μs)".to_string(),
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
                    .map(|r| r.avg_time())
                    .unwrap_or_default()
                    .as_micros()
                    .to_string(),
                results
                    .get(&part2)
                    .map(|r| r.avg_time())
                    .unwrap_or_default()
                    .as_micros()
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
