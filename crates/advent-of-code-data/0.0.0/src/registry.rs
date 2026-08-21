use std::collections::HashMap;

use thiserror::Error;

use crate::{Answer, Day, Year};

#[derive(Error, Debug)]
pub enum SolverError {
    #[error("there is not answer because the solver is not finished")]
    NotFinished,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = core::result::Result<T, SolverError>;
pub type SolverFn = fn(&str) -> Result<Answer>;

#[derive(Clone, Debug)]
pub struct Solver {
    pub day: Day,
    pub year: Year,
    pub part_one: SolverFn,
    pub part_two: SolverFn,
}

pub struct SolverRegistry {
    solvers: HashMap<Year, HashMap<Day, Solver>>,
}

impl SolverRegistry {
    pub fn new(all_solvers: &[Solver]) -> Self {
        let mut solvers: HashMap<Year, HashMap<Day, Solver>> = Default::default();

        for s in all_solvers.iter() {
            match solvers.get_mut(&s.year) {
                Some(solvers_for_year) => {
                    solvers_for_year.insert(s.day, s.clone());
                }
                None => {
                    solvers.insert(s.year, HashMap::from([(s.day, s.clone())]));
                }
            }
        }

        Self { solvers }
    }

    pub fn years(&self) -> Vec<Year> {
        self.solvers.keys().cloned().collect()
    }

    pub fn days(&self, _year: Year) -> Vec<Day> {
        todo!()
    }

    pub fn solver(&self, _day: Day, _yearr: Year) -> &Solver {
        todo!()
    }

    pub fn run_all(&self) {
        // TODO: move this out of registry and into CLI main.
        for solvers_for_year in self.solvers.values() {
            for s in solvers_for_year.values() {
                let a1 = (s.part_one)("");
                let a2 = (s.part_two)("");

                println!(
                    "day {}, year {}: part 1 = `{:#?}`, part 2 = `{:#?}`",
                    s.day, s.year, a1, a2
                );
            }
        }
    }
}
