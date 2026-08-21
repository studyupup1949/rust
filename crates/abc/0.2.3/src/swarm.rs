extern crate rand;

use self::rand::Rng;

use std::sync::{Mutex, RwLock, LockResult, Condvar};
use solution::Solution;
use row::Row;
use task::{TaskGenerator, Task};
use scaling::FitnessScaler;
use threaditer::Control;

// Completely ignore lock poisoning.
fn force_guard<Guard>(result: LockResult<Guard>) -> Guard {
    match result {
        Ok(x) => x,
        Err(err) => err.into_inner()
    }
}

pub struct Swarm<'a, S: Solution> {
    tasks: &'a Mutex<TaskGenerator>,
    rows: &'a [RwLock<Row<S>>],
    best: &'a Control<(f64, S)>,
    retries: usize,
    scale: &'a FitnessScaler,
}

impl<'a, S: Solution> Swarm<'a, S> {
    pub fn new(tasks: &'a Mutex<TaskGenerator>,
            rows: &'a [RwLock<Row<S>>],
            best: &'a Control<(f64, S)>,
            retries: usize,
            scale: &'a FitnessScaler) -> Swarm<'a, S> {
        Swarm {
            tasks: tasks,
            rows: rows,
            best: best,
            retries: retries,
            scale: scale,
        }
    }

    fn make_row(&self, solution: S) -> Row<S> {
        Row::new(solution, self.retries)
    }

    fn current_rows(&self) -> Vec<Row<S>> {
        self.rows.iter().map(|row_mutex| {
            let read_lock = force_guard(row_mutex.read());
            read_lock.clone()
        }).collect::<Vec<Row<S>>>()
    }

    fn work_on(&self, current_rows: &[Row<S>], n: usize) {
        let current_solution = &current_rows[n];
        let variant_solution = S::explore(current_rows, n);
        let variant_fitness = variant_solution.evaluate_fitness();

        // Check the cached fitness to see if we need bother getting a lock.
        if variant_fitness > current_solution.fitness {
            {
                let mut write_lock = force_guard(self.rows[n].write());

                // Make sure that the read row has not been replaced with
                // something better than our variant.
                if variant_fitness > write_lock.fitness {
                    *write_lock = self.make_row(variant_solution.clone());
                }
            }

            self.best.add_item((variant_fitness, variant_solution));
        } else {
            let mut write_lock = force_guard(self.rows[n].write());

            // Scouting has been folded into the working process
            if write_lock.expired() {
                let solution = S::make();
                *write_lock = self.make_row(solution);
            } else {
                write_lock.deplete();
            }
        }
    }

    fn choose(&self, current_rows: &[Row<S>], rng: &mut Rng) -> usize {
        let fitnesses = current_rows.iter()
            .map(|row| row.fitness)
            .collect::<Vec<f64>>();

        let fitnesses = &(self.scale)(fitnesses);

        let running_totals = fitnesses.iter().scan(0f64, |total, fitness| {
            *total += *fitness;
            Some(*total)
        }).collect::<Vec<f64>>();

        let total_fitness = running_totals.last().unwrap();
        let choice_point = rng.next_f64() * total_fitness;

        for (i, total) in running_totals.iter().enumerate() {
            if *total > choice_point {
                return i;
            }
        }
        unreachable!();
    }

    fn get_task(&self) -> Option<Task> {
        let mut task_cycle = force_guard(self.tasks.lock());
        task_cycle.next()
    }

    pub fn run(&self) {
        let mut rng = rand::thread_rng();
        while let Some(t) = self.get_task() {
            match t {
                Task::Worker(n) => {
                    let current_rows = self.current_rows();
                    self.work_on(&current_rows, n);
                }
                Task::Observer(_) => {
                    let current_rows = self.current_rows();
                    let index = self.choose(&current_rows, &mut rng);
                    self.work_on(&current_rows, index);
                }
            };
        }
    }
}