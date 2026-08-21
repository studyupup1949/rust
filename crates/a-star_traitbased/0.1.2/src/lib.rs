use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::rc::Rc;

pub trait PathGenerator {
    fn generate_paths(&self, from_position: (usize, usize)) -> Vec<(usize, usize)>;
    fn calculate_heuristic_cost(
        &self,
        position: (usize, usize),
        target: (Option<usize>, Option<usize>),
    ) -> usize;
    fn calculate_cost(
        &self,
        current_position: (usize, usize),
        next_position: (usize, usize),
    ) -> usize;
}

enum NextNodeResult<T> {
    Ok(T),
    Finished,
}

pub struct AStar {
    target: (Option<usize>, Option<usize>),
    que: Vec<Node>,
    closed_nodes: Vec<Rc<Node>>,
}

impl AStar {
    fn new(target: (Option<usize>, Option<usize>)) -> Self {
        Self {
            target,
            que: Vec::new(),
            closed_nodes: Vec::new(),
        }
    }

    pub fn run<T: PathGenerator>(
        from_struct: Box<&T>,
        start: (usize, usize),
        target: (Option<usize>, Option<usize>),
    ) -> Option<Vec<(usize, usize)>> {
        // PathGenerator is used to build possible paths
        let mut inst = Self::new(target);
        let exposed_struct = *from_struct;
        inst.que.push(Node::new(
            start,
            exposed_struct.calculate_heuristic_cost(start, target),
        ));
        loop {
            if inst.que.is_empty() {
                return None; // no elements left therefor no fast way out
            }
            inst.que.sort();
            let top = Rc::new(inst.que.remove(0));
            let possible_paths = exposed_struct.generate_paths(top.position);
            if possible_paths.len() != 0 {
                for possible_path in possible_paths {
                    if inst.pull_from_closed_by_position(possible_path).is_some() {
                        continue;
                    }
                    match inst.create_new_node(
                        Rc::clone(&top),
                        possible_path,
                        exposed_struct.calculate_cost(top.position, possible_path),
                        exposed_struct.calculate_heuristic_cost(possible_path, inst.target),
                    ) {
                        NextNodeResult::Ok(node) => inst.que.push(node),
                        NextNodeResult::Finished => {
                            let mut path = inst.reconstruct_path(Rc::clone(&top));
                            path.insert(0, possible_path);
                            return Some(path);
                        }
                    }
                }
            }
            inst.closed_nodes.push(Rc::clone(&top));
        }
    }

    fn create_new_node(
        &self,
        old_node: Rc<Node>,
        new_position: (usize, usize),
        cost: usize,
        heuristic_cost: usize,
    ) -> NextNodeResult<Node> {
        if self.target_is_reached(&new_position) {
            return NextNodeResult::Finished;
        }
        let new_cost = cost + old_node.cost;
        NextNodeResult::Ok(Node {
            position: new_position,
            comes_from: Some(old_node),
            cost: new_cost,
            total_cost: heuristic_cost + new_cost,
        })
    }

    fn target_is_reached(&self, position: &(usize, usize)) -> bool {
        if self.target.0.is_some() && self.target.0.unwrap() != position.0 {
            return false;
        }
        if self.target.1.is_some() && self.target.1.unwrap() != position.1 {
            return false;
        }
        true
    }

    fn reconstruct_path(&self, opt: Rc<Node>) -> Vec<(usize, usize)> {
        let mut fastest_path = vec![opt.position];
        let mut comes_from = opt.comes_from.as_ref();
        loop {
            if comes_from.is_some() {
                fastest_path.push(comes_from.unwrap().position);
            } else {
                return fastest_path;
            }
            comes_from = comes_from.unwrap().comes_from.as_ref();
        }
    }

    fn pull_from_closed_by_position(&self, position: (usize, usize)) -> Option<&Node> {
        for closed_node in self.closed_nodes.iter() {
            if closed_node.position == position {
                return Some(closed_node);
            }
        }
        None
    }
}

#[derive(Eq, Debug)]
struct Node {
    position: (usize, usize),
    cost: usize,
    total_cost: usize,
    comes_from: Option<Rc<Node>>,
}

impl Node {
    fn new(position: (usize, usize), total_cost: usize) -> Self {
        Self {
            position,
            comes_from: None,
            cost: 0,
            total_cost,
        }
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.total_cost == other.total_cost
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        self.total_cost.cmp(&other.total_cost)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }

    fn ge(&self, other: &Self) -> bool {
        self.total_cost >= other.total_cost
    }
    fn le(&self, other: &Self) -> bool {
        self.total_cost <= other.total_cost
    }
    fn gt(&self, other: &Self) -> bool {
        self.total_cost > other.total_cost
    }
    fn lt(&self, other: &Self) -> bool {
        self.total_cost < other.total_cost
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn testrun() {
        fn calc_usize_diff(x: usize, y: usize) -> usize {
            if x > y {
                return x - y;
            }
            y - x
        }

        struct Map {
            blocks: Vec<(usize, usize)>,
        }
        impl Map {
            fn path_is_possible(&self, possible_path: (usize, usize)) -> Option<(usize, usize)> {
                if self.blocks.contains(&possible_path) {
                    return None;
                }
                Some(possible_path)
            }
        }
        impl PathGenerator for Map {
            fn generate_paths(&self, from_position: (usize, usize)) -> Vec<(usize, usize)> {
                let mut possible_paths: Vec<(usize, usize)> = Vec::new();

                if from_position.0 != 0 && from_position.1 != 0 {
                    for possible_path in [
                        (from_position.0 - 1, from_position.1 - 1),
                        (from_position.0, from_position.1 - 1),
                        (from_position.0 - 1, from_position.1),
                    ] {
                        if let Some(path_) = self.path_is_possible(possible_path) {
                            possible_paths.push(path_)
                        }
                    }
                };
                for possible_path in [
                    (from_position.0 + 1, from_position.1 + 1),
                    (from_position.0, from_position.1 + 1),
                    (from_position.0 + 1, from_position.1),
                ] {
                    if let Some(path_) = self.path_is_possible(possible_path) {
                        possible_paths.push(path_)
                    }
                }
                return possible_paths;
            }
            #[allow(unused_variables)]
            fn calculate_cost(
                &self,
                current_position: (usize, usize),
                next_position: (usize, usize),
            ) -> usize {
                1
            }
            fn calculate_heuristic_cost(
                &self,
                position: (usize, usize),
                target: (Option<usize>, Option<usize>),
            ) -> usize {
                if target.0.is_none() && target.1.is_none() {
                    return 0;
                }
                if target.0.is_none() {
                    return calc_usize_diff(target.1.unwrap(), position.1);
                }
                if target.1.is_none() {
                    return calc_usize_diff(target.0.unwrap(), position.0);
                }
                return f64::sqrt(
                    ((calc_usize_diff(target.0.unwrap(), position.0) ^ 2)
                        + (calc_usize_diff(target.1.unwrap(), position.1) ^ 2))
                        as f64,
                ) as usize;
            }
        }

        let map_fixture = Map {
            blocks: vec![(2, 2)],
        };
        let path = AStar::run(Box::new(&map_fixture), (0, 0), (Some(3), Some(3)));
        assert_eq!(path.unwrap(), vec![(3, 3), (2, 3), (1, 2), (1, 1), (0, 0)])
    }
}
