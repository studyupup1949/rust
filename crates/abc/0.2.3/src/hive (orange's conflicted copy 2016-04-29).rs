extern crate num_cpus;

use task::TaskGenerator;
use row::Row;
use solution::Solution;
use scaling::{FitnessScaler, proportionate};
use swarm::Swarm;
use threaditer::{Control, ThreadIterator};

use std::thread::{JoinHandle, spawn};
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::sync::{Arc, Mutex, RwLock, Condvar};

struct Running {
    tasks: Arc<Mutex<TaskGenerator>>,
    handles: Vec<JoinHandle<()>>,
}

pub struct Hive<S: Solution> {
    workers: usize,
    observers: usize,
    retries: usize,
    threads: usize,
    rows: Arc<Vec<RwLock<Row<S>>>>,
    best: Arc<Control<(f64, S)>>,
    running: Option<Running>,
    scale: FitnessScaler,
    max_rounds: Option<usize>,
}

impl<S: Solution + Debug> Debug for Hive<S> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        for mutex in (&self.rows).iter() {
            let row = mutex.read().unwrap();
            try!(write!(f, "..{:?}..\n", *row));
        }
        let best_row = self.get();
        write!(f, ">>{:?}<<", best_row)
    }
}

impl<S: Solution> Hive<S> {
    pub fn new(workers: usize, observers: usize, retries: usize) -> Hive<S> {
        if workers == 0 {
            panic!("Hive must have at least one worker.");
        }

        let mut best_pair = None;
        let mut rows = Vec::with_capacity(workers);

        for _ in 0..workers {
            let solution = S::make();
            let row = Row::new(solution, retries);
            best_pair = {
                let new_candidate = Some((row.fitness, row.solution.clone()));
                match best_pair {
                    None => new_candidate,
                    Some((best_fitness, best_solution)) => {
                        if row.fitness > best_fitness {
                            new_candidate
                        } else {
                            Some((best_fitness, best_solution))
                        }
                    }
                }
            };
            rows.push(RwLock::new(row))
        }

        let mut best = Control::new();
        best.set_policy(Box::new(|current, new| {
            current.as_ref().map_or(true, |&(current_fitness, _)|
                new.as_ref().map_or(true, |&(new_fitness, _)|
                    new_fitness > current_fitness))
        }));
        best.add_item(best_pair.unwrap());

        Hive {
            workers: workers,
            observers: observers,
            retries: retries,
            threads: num_cpus::get(),
            rows: Arc::new(rows),
            best: Arc::new(best),
            scale: proportionate(),
            running: None,
            max_rounds: None,
        }
    }

    pub fn set_threads(mut self, threads: usize) -> Hive<S> {
        self.threads = threads;
        self
    }

    pub fn set_scaling(mut self, scale: FitnessScaler) -> Hive<S> {
        self.scale = scale;
        self
    }

    pub fn set_max_rounds(mut self, max_rounds: usize) -> Hive<S> {
        self.max_rounds = Some(max_rounds);
        self
    }

    pub fn stop(&mut self) {
        if let Some(Running {tasks, mut handles}) = self.running.take() {
            {
                let mut tasks = tasks.lock().unwrap();
                tasks.stop();
            }
            for handle in handles.drain(..) {
                handle.join().unwrap();
            }
        }
    }

    pub fn run(&mut self) {
        if self.running.is_none() {
            let cycle = TaskGenerator::new(self.workers, self.observers, self.max_rounds);
            let tasks = Arc::new(Mutex::new(cycle));

            let join_handles: Vec<_> = (0..self.threads).map(|_| {
                let tasks = tasks.clone();
                let rows = self.rows.clone();
                let best = self.best.clone();
                let retries = self.retries; // remove dependence on self borrow
                let scale = self.scale.clone();
                spawn(move|| Swarm::new(&tasks, &rows, &best, retries, &scale).run())
            }).collect();

            self.running = Some(Running {
                tasks: tasks.clone(),
                handles: join_handles,
            });
        }
    }

    pub fn get(&self) -> (f64, S) {
        self.best.lock().clone().unwrap()
    }

    pub fn get_round(&mut self) -> usize {
        // We need to briefly acquire self.running, so we are technically
        // mutating self, though we restore it right afterwards. Since we
        // have a mutable reference to self, we don't anticipate problems.
        if let Some(Running { tasks, handles }) = self.running.take() {
            let round = tasks.lock().unwrap().round;
            self.running = Some(Running { tasks: tasks, handles: handles });
            round
        } else {
            0
        }
    }

    pub fn stream<'a>(&'a mut self) -> ThreadIterator<(f64, S)> {
        self.run();
        ThreadIterator::new(self.best.clone())
    }
}

impl<S: Solution> Drop for Hive<S> {
    fn drop(&mut self) {
        self.stop();
    }
}