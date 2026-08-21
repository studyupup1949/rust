use rug::Integer;
use std::collections::HashSet;

/// [Trellis] is a data structure to hold a bounded trellis
///
/// Trellis nodes hold a [rug::Integer] and are indexed by `stage` (0..n_max)
/// and `weight_level` (one of the accepted weight levels).
///
/// `weight_levels` for each `stage` are returned by [Trellis::get_weight_levels()].
/// Node values can be read and set by using the [Trellis::get()] and [Trellis::set()]
/// methods.
#[derive(Debug)]
pub struct Trellis {
    pub threshold: usize,
    pub n_max: usize,
    weights: Vec<usize>,
    weight_levels: Vec<usize>,
    weight_level_lookup: Vec<i64>,
    sorted_weights: Vec<(usize, usize)>,
    data: Vec<Vec<Integer>>,
}

impl Trellis {
    /// Create a new [Trellis] instance
    ///
    /// The smallest weight must be 0
    pub fn new(threshold: usize, n_max: usize, weights: &[usize]) -> Trellis {
        assert_eq!(*weights.iter().min().unwrap(), 0);

        let mut sorted_weights: Vec<(usize, usize)> = weights.iter().copied().enumerate().collect();
        sorted_weights.sort_by_key(|&w_tuple| w_tuple.1);

        let weight_levels = Trellis::calc_weight_levels(threshold, weights);
        let weight_level_lookup = Trellis::make_weight_level_lookup(&weight_levels);

        let data = vec![vec![Integer::from(0); weight_levels.len()]; 1 + n_max];

        Trellis {
            threshold,
            n_max,
            weights: weights.to_vec(),
            weight_levels,
            weight_level_lookup,
            sorted_weights,
            data,
        }
    }

    pub fn new_like(trellis: &Trellis) -> Trellis {
        Trellis::new(trellis.threshold, trellis.n_max, &trellis.get_weights())
    }

    pub fn new_expandable(n_max: usize, weights: &[usize]) -> Trellis {
        assert_eq!(*weights.iter().min().unwrap(), 0);

        let mut sorted_weights: Vec<(usize, usize)> = weights.iter().copied().enumerate().collect();
        sorted_weights.sort_by_key(|&w_tuple| w_tuple.1);

        let max_weight = weights
            .iter()
            .max()
            .expect("Already checked if empty in assert above");
        let max_threshold = n_max * max_weight;
        let all_wls = Trellis::calc_weight_levels(max_threshold, weights);
        let wl_lookup = Trellis::make_weight_level_lookup(&all_wls);

        let data = vec![Vec::<Integer>::new(); 1 + n_max];

        let threshold = all_wls[0];

        Trellis {
            threshold,
            n_max,
            weights: weights.to_vec(),
            weight_levels: all_wls,
            weight_level_lookup: wl_lookup,
            sorted_weights,
            data,
        }
    }

    fn calc_weight_levels(threshold: usize, weights: &[usize]) -> Vec<usize> {
        let mut weight_levels = HashSet::new();
        weight_levels.insert(0);

        let mut new_exist = true;
        //print!("Calculating weight levels ");
        while new_exist {
            new_exist = false;
            let mut new_entries = vec![];
            for wl in weight_levels.iter() {
                for w in weights.iter() {
                    let new_wl = wl + w;
                    if new_wl <= threshold {
                        if !weight_levels.contains(&new_wl) {
                            new_exist = true;
                        }
                        new_entries.push(new_wl);
                    }
                }
            }
            for new_wl in new_entries.into_iter() {
                weight_levels.insert(new_wl);
            }
            //print!(".");
        }

        // convert to sorted vec
        let mut weight_levels: Vec<usize> = weight_levels.into_iter().collect();
        weight_levels.sort();

        //println!(" done");
        weight_levels
    }

    fn make_weight_level_lookup(weight_levels: &[usize]) -> Vec<i64> {
        let max_wl = weight_levels
            .iter()
            .max()
            .expect("weight_levels must be non empty");
        let mut wl_lookup = vec![-1; *max_wl + 1];
        for (wl_idx, &wl) in weight_levels.iter().enumerate() {
            wl_lookup[wl] = wl_idx as i64;
        }
        wl_lookup
    }
    fn wl_idx_valid(weight_level_index: i64) -> bool {
        // use not negative as 0 is a valid index
        !weight_level_index.is_negative()
    }
}

impl Trellis {
    fn wl_valid(&self, weight_level: usize) -> bool {
        let weight_level_index = self.weight_level_lookup[weight_level];
        Trellis::wl_idx_valid(weight_level_index)
    }
    /// Get function for trellis values
    pub fn get(&self, stage: usize, weight_level: usize) -> Integer {
        let weight_level_index = self.weight_level_lookup[weight_level];
        assert!(Trellis::wl_idx_valid(weight_level_index));
        self.data[stage][weight_level_index as usize].clone()
    }
    /// Get function for trellis values, returns 0 if `weight_level` is invalid
    pub fn get_or_0(&self, stage: usize, weight_level: usize) -> Integer {
        if weight_level >= self.weight_level_lookup.len() {
            return Integer::from(0);
        }
        let weight_level_index = self.weight_level_lookup[weight_level];

        if Trellis::wl_idx_valid(weight_level_index) {
            self.data[stage][weight_level_index as usize].clone()
        } else {
            Integer::from(0)
        }
    }
    pub fn get_stage(&self, stage: usize) -> Vec<Integer> {
        self.data[stage].clone()
    }
    /// Set function for trellis values
    pub fn set(&mut self, stage: usize, weight_level: usize, value: Integer) {
        let weight_level_index = self.weight_level_lookup[weight_level];
        assert!(Trellis::wl_idx_valid(weight_level_index));
        self.data[stage][weight_level_index as usize] = value;
    }
    /// Function to add a value to an existing trellis value
    pub fn add(&mut self, stage: usize, weight_level: usize, value: Integer) {
        let weight_level_index = self.weight_level_lookup[weight_level];
        assert!(Trellis::wl_idx_valid(weight_level_index));
        self.data[stage][weight_level_index as usize] += value;
    }
    /// Returns the weight for the given weight index
    pub fn get_weight(&self, weight_index: usize) -> usize {
        self.weights[weight_index]
    }
    /// Returns the weights of this trellis
    pub fn get_weights(&self) -> Vec<usize> {
        self.weights.clone()
    }
    /// Returns the weight levels of this trellis
    pub fn get_weight_levels(&self) -> Vec<usize> {
        self.weight_levels.clone()
    }
    /// Returns the number of weight levels used by the stored data
    pub fn get_num_weight_levels(&self) -> usize {
        self.data[0].len()
    }
    /// Returns the index of the given weight level
    pub fn get_weight_level_index(&self, weight_level: usize) -> usize {
        let weight_level_index = self.weight_level_lookup[weight_level];
        assert!(Trellis::wl_idx_valid(weight_level_index));
        weight_level_index as usize
    }
    pub fn get_storage_dimensions(&self) -> (usize, usize) {
        (self.data.len(), self.get_num_weight_levels())
    }
    /// Increase the trellis size by one weight level mooving in the provided trellis values
    ///
    /// Note: the values are removed from `new_values`
    pub fn expand_with(&mut self, new_values: &mut Vec<Integer>) -> Result<(), &'static str> {
        assert_eq!(new_values.len(), self.data.len());

        let current_num_wls = self.get_num_weight_levels();
        let new_num_wls = current_num_wls + 1;

        let max_num_wls = self.weight_levels.len();
        if new_num_wls <= max_num_wls {
            for stage in self.data.iter_mut().rev() {
                stage.push(new_values.pop().expect("checked lenghts in assert"))
            }
            self.threshold = self.weight_levels[new_num_wls - 1];
            Ok(())
        } else {
            Err("Impossible to add another weight level, trellis to small")
        }
    }
    /// Returns a [Vec] of (weight_index, weight_level) for each weight level reachable
    /// from `weight_level` with a single step
    ///
    /// The weight levels are sorted in ascending order.
    /// Multiple entries with the same weight level are sorted by weight index in ascending order.
    pub fn get_successors(&self, weight_level: usize) -> Vec<(usize, usize)> {
        let mut successors = Vec::with_capacity(self.weights.len());
        for (weight_index, w) in self.sorted_weights.iter() {
            let possible_successor = weight_level + w;
            if possible_successor <= self.threshold {
                successors.push((*weight_index, possible_successor));
            }
        }
        successors
    }
    /// Returns a [Vec] of (weight_index, weight_level) for each weight level which
    /// can reach `weight_level` with a single step
    ///
    /// The tuples are sorted in ascending order wrt. the weight_level values.
    /// Multiple tuples with the same `weight_level` are sorted in descending order wrt. the
    /// `weight_index`.
    pub fn get_predecessors(&self, weight_level: usize) -> Vec<(usize, usize)> {
        let mut predecessors = Vec::with_capacity(self.weights.len());
        for (weight_index, w) in self.sorted_weights.iter().rev() {
            if weight_level >= *w {
                let possible_predecessor = weight_level - w;
                if self.wl_valid(possible_predecessor) {
                    predecessors.push((*weight_index, possible_predecessor));
                }
            }
        }
        predecessors
    }
}

impl PartialEq for Trellis {
    fn eq(&self, other: &Self) -> bool {
        if self.get_storage_dimensions() != other.get_storage_dimensions()
            || self.get_weights() != other.get_weights()
        {
            return false;
        }
        let (i_max, j_max) = self.get_storage_dimensions();
        for i in 0..i_max {
            for j in 0..j_max {
                if self.data[i][j] != other.data[i][j] {
                    return false;
                }
            }
        }
        true
    }
}

impl Eq for Trellis {}
