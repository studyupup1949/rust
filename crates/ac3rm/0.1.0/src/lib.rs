use crate::{
    constraint::ConstraintEntry,
    var::{ValueState, Variable},
};
pub use constraint::{Constraint, ConstraintId};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt,
};
use tracing::trace;
pub use var::VarId;

mod constraint;
mod var;

type Callback = Box<dyn Fn(VarId)>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropagationError {
    InvalidConstraintId(ConstraintId),
    DomainWipeout { var: VarId, explanation: Vec<ConstraintId> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Arc {
    constraint_id: ConstraintId,
    from: VarId,
    to: VarId,
}

pub struct Engine {
    variables: Vec<Variable>,
    constraints: Vec<ConstraintEntry>,
    // Key: (constraint_id, from_var, to_var, from_value) -> supporting to_value.
    residues: HashMap<(ConstraintId, VarId, VarId, i32), i32>,
    listeners: HashMap<VarId, Vec<Callback>>,
    // Key: (var, value) -> set of constraint IDs that transitively explain why the value is killed.
    prune_reasons: HashMap<(VarId, i32), HashSet<ConstraintId>>,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    /// Creates a new empty constraint engine with no variables or constraints.
    pub fn new() -> Self {
        Self {
            variables: Vec::new(),
            constraints: Vec::new(),
            residues: HashMap::new(),
            listeners: HashMap::new(),
            prune_reasons: HashMap::new(),
        }
    }

    /// Adds a new variable with the specified domain.
    ///
    /// All values in the domain start as active (not suppressed by any constraint).
    /// Duplicate values are automatically deduplicated.
    ///
    /// Returns the variable ID, which can be used with `new_eq`, `new_neq`, `set`, and `forbid`.
    ///
    /// # Example
    /// ```
    /// let mut engine = ac3rm::Engine::new();
    /// let x = engine.add_var([1, 2, 3]);
    /// assert_eq!(engine.val(x), vec![1, 2, 3]);
    /// ```
    pub fn add_var(&mut self, domain: impl IntoIterator<Item = i32>) -> VarId {
        let mut unique = Vec::new();
        let mut seen = HashSet::new();

        for value in domain {
            if seen.insert(value) {
                unique.push(value);
            }
        }

        let mut index_by_value = HashMap::with_capacity(unique.len());
        let mut states = Vec::with_capacity(unique.len());

        for (idx, value) in unique.into_iter().enumerate() {
            index_by_value.insert(value, idx);
            states.push(ValueState { value, killers: HashSet::new() });
        }

        let id = self.variables.len();
        trace!("Adding variable e{} with domain {{{}}}", id, states.iter().map(|s| s.value.to_string()).collect::<Vec<_>>().join(", "));
        self.variables.push(Variable { domain: states, index_by_value });
        VarId(id)
    }

    /// Returns the currently active domain values of a variable.
    ///
    /// Values that are suppressed by active constraints are excluded from the result.
    ///
    /// # Panics
    /// Panics if `var_id` is not a valid variable ID.
    pub fn val(&self, var_id: VarId) -> Vec<i32> {
        self.variables[var_id.0].domain.iter().filter_map(|state| if state.killers.is_empty() { Some(state.value) } else { None }).collect()
    }

    /// Adds a constraint to the engine without activating it.
    ///
    /// Returns the constraint ID. Use `assert` to activate it and trigger propagation.
    pub fn new_constraint(&mut self, constraint: Constraint) -> ConstraintId {
        let id = self.constraints.len();
        trace!("Adding constraint c{}: {}", id, constraint);
        self.constraints.push(ConstraintEntry { active: false, kind: constraint });
        ConstraintId(id)
    }

    /// Activates a constraint and propagates its effects.
    ///
    /// If the constraint is already active, this is a no-op.
    ///
    /// # Errors
    /// Returns `PropagationError` if propagation causes a domain wipeout (no solution exists).
    pub fn assert(&mut self, constraint_id: ConstraintId) -> Result<(), PropagationError> {
        trace!("Activating constraint c{}", self.constraints[constraint_id.0].kind);
        let touched = self.constraint_vars(constraint_id)?;
        if self.constraints[constraint_id.0].active {
            return Ok(());
        }

        self.constraints[constraint_id.0].active = true;
        self.propagate_from_vars(&touched)
    }

    /// Deactivates a constraint and re-propagates the affected neighborhood.
    ///
    /// If the constraint is already inactive, this is a no-op.
    /// Values that were suppressed only by this constraint are restored,
    /// and the affected subgraph is re-propagated incrementally.
    ///
    /// # Errors
    /// Returns `PropagationError` if re-propagation unexpectedly causes a domain wipeout.
    /// This should not happen in normal operation.
    pub fn retract(&mut self, constraint_id: ConstraintId) -> Result<(), PropagationError> {
        trace!("Deactivating constraint c{}", self.constraints[constraint_id.0].kind);
        let touched = self.constraint_vars(constraint_id)?;
        if !self.constraints[constraint_id.0].active {
            return Ok(());
        }

        self.constraints[constraint_id.0].active = false;

        let mut restored: Vec<(VarId, i32)> = Vec::new();
        for &var in &touched {
            for state in &mut self.variables[var.0].domain {
                let was_inactive = !state.killers.is_empty();
                state.killers.remove(&constraint_id);
                if was_inactive && state.killers.is_empty() {
                    restored.push((var, state.value));
                }
            }
        }
        for key in restored {
            self.prune_reasons.remove(&key);
        }

        self.residues.retain(|(cid, _, _, _), _| *cid != constraint_id);
        self.propagate_from_vars(&touched)
    }

    /// Creates an equality constraint between two variables and asserts it.
    ///
    /// The two variables must have at least one common value, or propagation will fail.
    ///
    /// # Errors
    /// Returns `PropagationError::DomainWipeout` if no common value exists.
    ///
    /// # Example
    /// ```
    /// # fn main() -> Result<(), ac3rm::PropagationError> {
    /// let mut engine = ac3rm::Engine::new();
    /// let a = engine.add_var([1, 2, 3]);
    /// let b = engine.add_var([2, 3, 4]);
    /// engine.add_eq(a, b)?;  // Both now have domain {2, 3}
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_eq(&mut self, a: VarId, b: VarId) -> Result<ConstraintId, PropagationError> {
        let id = self.new_constraint(Constraint::Equality(a, b));
        self.assert(id)?;
        Ok(id)
    }

    /// Creates an inequality constraint between two variables and asserts it.
    ///
    /// # Errors
    /// Returns `PropagationError::DomainWipeout` if a domain becomes empty.
    pub fn add_neq(&mut self, a: VarId, b: VarId) -> Result<ConstraintId, PropagationError> {
        let id = self.new_constraint(Constraint::Inequality(a, b));
        self.assert(id)?;
        Ok(id)
    }

    /// Creates a unary set constraint (variable must equal value) and asserts it.
    ///
    /// # Errors
    /// Returns `PropagationError::DomainWipeout` if the value is not in the variable's domain.
    pub fn set(&mut self, var: VarId, value: i32) -> Result<ConstraintId, PropagationError> {
        let id = self.new_constraint(Constraint::Set(var, value));
        self.assert(id)?;
        Ok(id)
    }

    /// Creates a unary forbid constraint (variable cannot equal value) and asserts it.
    ///
    /// # Errors
    /// Returns `PropagationError::DomainWipeout` if the value is the only value in the domain.
    pub fn forbid(&mut self, var: VarId, value: i32) -> Result<ConstraintId, PropagationError> {
        let id = self.new_constraint(Constraint::Forbid(var, value));
        self.assert(id)?;
        Ok(id)
    }

    /// Asserts multiple constraints at once and propagates them together.
    ///
    /// More efficient than calling `assert` multiple times, as it accumulates
    /// all affected variables and performs a single propagation pass.
    /// Automatically deduplicates variables using a `HashSet`.
    ///
    /// # Errors
    /// Returns `PropagationError` if any assertion fails or propagation causes a domain wipeout.
    ///
    /// # Example
    /// ```
    /// # fn main() -> Result<(), ac3rm::PropagationError> {
    /// use ac3rm::Constraint;
    /// let mut engine = ac3rm::Engine::new();
    /// let x = engine.add_var([1, 2, 3]);
    /// let y = engine.add_var([2, 3, 4]);
    /// let id1 = engine.new_constraint(Constraint::Equality(x, y));
    /// let id2 = engine.new_constraint(Constraint::Set(x, 2));
    /// engine.assert_batch(&[id1, id2])?;  // Single propagation pass
    /// # Ok(())
    /// # }
    /// ```
    pub fn assert_batch(&mut self, constraint_ids: &[ConstraintId]) -> Result<(), PropagationError> {
        let mut all_touched = HashSet::new();

        for &id in constraint_ids {
            trace!("Activating constraint c{}", self.constraints[id.0].kind);
            let touched = self.constraint_vars(id)?;
            if !self.constraints[id.0].active {
                self.constraints[id.0].active = true;
                all_touched.extend(touched);
            }
        }

        self.propagate_from_vars(&all_touched.into_iter().collect::<Vec<_>>())
    }

    /// Retracts multiple constraints at once and re-propagates incrementally.
    ///
    /// More efficient than calling `retract` multiple times, as it accumulates
    /// all affected variables and performs a single re-propagation pass.
    ///
    /// # Errors
    /// Returns `PropagationError` if re-propagation causes an unexpected domain wipeout.
    pub fn retract_batch(&mut self, constraint_ids: &[ConstraintId]) -> Result<(), PropagationError> {
        let mut all_touched = HashSet::new();
        let mut restored: Vec<(VarId, i32)> = Vec::new();

        for &id in constraint_ids {
            trace!("Deactivating constraint c{}", self.constraints[id.0].kind);
            let touched = self.constraint_vars(id)?;
            if self.constraints[id.0].active {
                self.constraints[id.0].active = false;

                for &var in &touched {
                    for state in &mut self.variables[var.0].domain {
                        let was_inactive = !state.killers.is_empty();
                        state.killers.remove(&id);
                        if was_inactive && state.killers.is_empty() {
                            restored.push((var, state.value));
                        }
                    }
                }

                all_touched.extend(touched);
            }
        }

        for key in restored {
            self.prune_reasons.remove(&key);
        }

        self.residues.retain(|(cid, _, _, _), _| !constraint_ids.contains(cid));
        self.propagate_from_vars(&all_touched.into_iter().collect::<Vec<_>>())
    }

    /// Registers a callback to be notified when a variable's domain changes.
    ///
    /// The callback receives the variable ID and is invoked whenever the domain
    /// is modified (values suppressed or restored) during constraint propagation.
    ///
    /// Multiple callbacks can be registered for the same variable.
    ///
    /// # Example
    /// ```
    /// let mut engine = ac3rm::Engine::new();
    /// let x = engine.add_var([1, 2, 3]);
    /// engine.set_listener(x, |var_id| {
    ///     println!("Variable {} domain changed", var_id);
    /// });
    /// ```
    pub fn set_listener<F>(&mut self, var: VarId, callback: F)
    where
        F: Fn(VarId) + 'static,
    {
        self.listeners.entry(var).or_default().push(Box::new(callback));
    }

    fn notify_listeners(&self, var: VarId) {
        if let Some(cbs) = self.listeners.get(&var) {
            for cb in cbs {
                cb(var);
            }
        }
    }

    fn constraint_vars(&self, constraint_id: ConstraintId) -> Result<Vec<VarId>, PropagationError> {
        let Some(entry) = self.constraints.get(constraint_id.0) else {
            return Err(PropagationError::InvalidConstraintId(constraint_id));
        };

        let vars = match entry.kind {
            Constraint::Equality(a, b) | Constraint::Inequality(a, b) => {
                if a == b {
                    vec![a]
                } else {
                    vec![a, b]
                }
            }
            Constraint::Set(var, _) | Constraint::Forbid(var, _) => vec![var],
        };

        Ok(vars)
    }

    fn arcs_of(&self, constraint_id: ConstraintId) -> Vec<Arc> {
        match self.constraints[constraint_id.0].kind {
            Constraint::Equality(a, b) | Constraint::Inequality(a, b) => {
                if a == b {
                    vec![Arc { constraint_id, from: a, to: b }]
                } else {
                    vec![Arc { constraint_id, from: a, to: b }, Arc { constraint_id, from: b, to: a }]
                }
            }
            Constraint::Set(var, _) | Constraint::Forbid(var, _) => {
                vec![Arc { constraint_id, from: var, to: var }]
            }
        }
    }

    fn touching_constraints(&self, var: VarId) -> Vec<ConstraintId> {
        self.constraints
            .iter()
            .enumerate()
            .filter_map(|(id, entry)| {
                if !entry.active {
                    return None;
                }

                let touches = match entry.kind {
                    Constraint::Equality(a, b) | Constraint::Inequality(a, b) => a == var || b == var,
                    Constraint::Set(v, _) | Constraint::Forbid(v, _) => v == var,
                };

                if touches { Some(ConstraintId(id)) } else { None }
            })
            .collect()
    }

    fn propagate_from_vars(&mut self, vars: &[VarId]) -> Result<(), PropagationError> {
        trace!("Starting propagation from variables: {:?}", vars);
        let mut queue = VecDeque::new();
        let mut in_queue = HashSet::new();

        for &var in vars {
            for cid in self.touching_constraints(var) {
                for arc in self.arcs_of(cid) {
                    if in_queue.insert(arc) {
                        queue.push_back(arc);
                    }
                }
            }
        }

        while let Some(arc) = queue.pop_front() {
            trace!("Processing arc: {} (from e{} to e{})", self.constraints[arc.constraint_id.0].kind, arc.from, arc.to);
            in_queue.remove(&arc);

            if !self.constraints[arc.constraint_id.0].active {
                continue;
            }

            let changed = self.revise(arc)?;
            if changed {
                // Re-queue only incoming arcs: Y_i -> X_j where X_j = arc.from
                for cid in self.touching_constraints(arc.from) {
                    for next_arc in self.arcs_of(cid) {
                        if next_arc.to == arc.from && next_arc != arc && in_queue.insert(next_arc) {
                            queue.push_back(next_arc);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn revise(&mut self, arc: Arc) -> Result<bool, PropagationError> {
        match self.constraints[arc.constraint_id.0].kind {
            Constraint::Set(_, expected) => self.revise_unary(arc.constraint_id, arc.from, |a| a == expected),
            Constraint::Forbid(_, forbidden) => self.revise_unary(arc.constraint_id, arc.from, |a| a != forbidden),
            Constraint::Equality(_, _) => self.revise_binary(arc.constraint_id, arc.from, arc.to, |a, b| a == b),
            Constraint::Inequality(_, _) => self.revise_binary(arc.constraint_id, arc.from, arc.to, |a, b| a != b),
        }
    }

    fn revise_unary<F>(&mut self, cid: ConstraintId, var: VarId, predicate: F) -> Result<bool, PropagationError>
    where
        F: Fn(i32) -> bool,
    {
        let mut changed = false;
        let mut newly_killed: Vec<i32> = Vec::new();
        let mut newly_restored: Vec<i32> = Vec::new();

        for state in &mut self.variables[var.0].domain {
            let was_active = state.killers.is_empty();
            if predicate(state.value) {
                state.killers.remove(&cid);
            } else {
                state.killers.insert(cid);
            }

            let is_active = state.killers.is_empty();
            if was_active != is_active {
                changed = true;
                if is_active {
                    newly_restored.push(state.value);
                } else {
                    newly_killed.push(state.value);
                }
            }
        }

        for &v in &newly_killed {
            self.prune_reasons.insert((var, v), std::iter::once(cid).collect());
        }
        for &v in &newly_restored {
            self.prune_reasons.remove(&(var, v));
        }

        if !self.has_active_value(var) {
            return Err(self.wipeout(var));
        }

        if changed {
            trace!("Variable e{} domain is now {{{}}}", var, self.variables[var.0].domain.iter().filter(|s| s.killers.is_empty()).map(|s| s.value.to_string()).collect::<Vec<_>>().join(", "));
            self.notify_listeners(var);
        }

        Ok(changed)
    }

    fn revise_binary<F>(&mut self, cid: ConstraintId, from: VarId, to: VarId, relation: F) -> Result<bool, PropagationError>
    where
        F: Fn(i32, i32) -> bool,
    {
        let mut changed = false;

        let from_values: Vec<i32> = self.variables[from.0].domain.iter().map(|s| s.value).collect();

        for a in from_values {
            let has_support = self.has_support(cid, from, to, a, &relation);

            // Compute transitive reason before mutating domain.
            // For each killed value in `to` that would have supported `a`, include its reason.
            let new_reason: Option<HashSet<ConstraintId>> = if !has_support {
                let mut reason = HashSet::new();
                reason.insert(cid);
                for state in &self.variables[to.0].domain {
                    if !state.killers.is_empty() && relation(a, state.value) {
                        if let Some(r) = self.prune_reasons.get(&(to, state.value)) {
                            reason.extend(r.iter().copied());
                        }
                    }
                }
                Some(reason)
            } else {
                None
            };

            let idx = *self.variables[from.0].index_by_value.get(&a).unwrap();
            let was_active = self.variables[from.0].domain[idx].killers.is_empty();

            if has_support {
                self.variables[from.0].domain[idx].killers.remove(&cid);
            } else {
                self.variables[from.0].domain[idx].killers.insert(cid);
            }

            let is_active = self.variables[from.0].domain[idx].killers.is_empty();
            if was_active != is_active {
                changed = true;
            }

            if let Some(reason) = new_reason {
                self.prune_reasons.insert((from, a), reason);
            } else if is_active {
                // Value restored (or still active): remove any stale reason.
                self.prune_reasons.remove(&(from, a));
            }
        }

        if !self.has_active_value(from) {
            return Err(self.wipeout(from));
        }

        if changed {
            trace!("Variable e{} domain is now {{{}}}", from, self.variables[from.0].domain.iter().filter(|s| s.killers.is_empty()).map(|s| s.value.to_string()).collect::<Vec<_>>().join(", "));
            self.notify_listeners(from);
        }

        Ok(changed)
    }

    fn has_support<F>(&mut self, cid: ConstraintId, from: VarId, to: VarId, a: i32, relation: &F) -> bool
    where
        F: Fn(i32, i32) -> bool,
    {
        let residue_key = (cid, from, to, a);

        if let Some(&b) = self.residues.get(&residue_key)
            && self.is_active_value(to, b)
            && relation(a, b)
        {
            return true;
        }

        for state in &self.variables[to.0].domain {
            if !state.killers.is_empty() {
                continue;
            }
            if relation(a, state.value) {
                self.residues.insert(residue_key, state.value);
                return true;
            }
        }

        self.residues.remove(&residue_key);
        false
    }

    fn is_active_value(&self, var: VarId, value: i32) -> bool {
        let Some(&idx) = self.variables[var.0].index_by_value.get(&value) else {
            return false;
        };
        self.variables[var.0].domain[idx].killers.is_empty()
    }

    fn has_active_value(&self, var: VarId) -> bool {
        self.variables[var.0].domain.iter().any(|s| s.killers.is_empty())
    }

    fn wipeout(&self, var: VarId) -> PropagationError {
        let mut explanation = HashSet::new();
        for state in &self.variables[var.0].domain {
            if !state.killers.is_empty() {
                // Use transitive reason if available; fall back to direct killers.
                if let Some(reason) = self.prune_reasons.get(&(var, state.value)) {
                    explanation.extend(reason.iter().copied());
                } else {
                    explanation.extend(state.killers.iter().copied());
                }
            }
        }

        PropagationError::DomainWipeout { var, explanation: explanation.into_iter().collect() }
    }
}

impl fmt::Display for Engine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Variables:")?;
        for (i, var) in self.variables.iter().enumerate() {
            let values: Vec<String> = var.domain.iter().map(|s| if s.killers.is_empty() { s.value.to_string() } else { format!("{} (killed by {:?})", s.value, s.killers) }).collect();
            writeln!(f, "  e{}: {}", i, values.join(", "))?;
        }

        writeln!(f, "Constraints:")?;
        for (i, entry) in self.constraints.iter().enumerate() {
            writeln!(f, "  c{}: {} [{}]", i, entry.kind, if entry.active { "active" } else { "inactive" })?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dynamic_unary_retract_and_readd() {
        let mut ac = Engine::new();
        let x = ac.add_var([1, 2, 3]);

        let c = ac.new_constraint(Constraint::Forbid(x, 2));
        ac.assert(c).expect("forbid must propagate");
        assert_eq!(ac.val(x), vec![1, 3]);

        ac.retract(c).expect("retraction must succeed");
        assert_eq!(ac.val(x), vec![1, 2, 3]);

        ac.assert(c).expect("re-assert must propagate");
        assert_eq!(ac.val(x), vec![1, 3]);
    }

    #[test]
    fn dynamic_binary_retract_and_readd() {
        let mut ac = Engine::new();
        let a = ac.add_var([1, 2, 3]);
        let b = ac.add_var([2, 3, 4]);

        let eq = ac.new_constraint(Constraint::Equality(a, b));
        ac.assert(eq).expect("equality must propagate");
        assert_eq!(ac.val(a), vec![2, 3]);
        assert_eq!(ac.val(b), vec![2, 3]);

        ac.retract(eq).expect("retraction must succeed");
        assert_eq!(ac.val(a), vec![1, 2, 3]);
        assert_eq!(ac.val(b), vec![2, 3, 4]);

        ac.assert(eq).expect("re-assert must propagate");
        assert_eq!(ac.val(a), vec![2, 3]);
        assert_eq!(ac.val(b), vec![2, 3]);
    }

    #[test]
    fn mixed_constraints_and_selective_retraction() {
        let mut ac = Engine::new();
        let a = ac.add_var([1, 2, 3]);
        let b = ac.add_var([1, 2, 3]);

        let eq = ac.add_eq(a, b).expect("eq must succeed");
        let set = ac.set(a, 2).expect("set must succeed");
        assert_eq!(ac.val(a), vec![2]);
        assert_eq!(ac.val(b), vec![2]);

        ac.retract(set).expect("retract set must succeed");
        assert_eq!(ac.val(a), vec![2]);
        assert_eq!(ac.val(b), vec![2]);

        ac.retract(eq).expect("retract eq must succeed");
        assert_eq!(ac.val(a), vec![1, 2, 3]);
        assert_eq!(ac.val(b), vec![1, 2, 3]);
    }

    #[test]
    fn test_basic_equality() {
        let mut ac = Engine::new();
        let a = ac.add_var([1, 2, 3]);
        let b = ac.add_var([2, 3, 4]);

        ac.add_eq(a, b).expect("equality must succeed");

        // Intersection should be {2, 3}
        assert_eq!(ac.val(a), vec![2, 3]);
        assert_eq!(ac.val(b), vec![2, 3]);
    }

    #[test]
    fn test_inequality_singleton_pruning() {
        let mut ac = Engine::new();
        let a = ac.add_var([1]);
        let b = ac.add_var([1, 2, 3]);

        ac.add_neq(a, b).expect("inequality must succeed");

        // Since a is {1}, b cannot be 1.
        assert_eq!(ac.val(b), vec![2, 3]);
    }

    #[test]
    fn test_multiple_suppression_logic() {
        let mut ac = Engine::new();
        let a = ac.add_var([1, 2, 3]);
        let b = ac.add_var([1]);
        let c = ac.add_var([1]);

        // Constraint 0: a != b  => a: {2, 3}
        let id0 = ac.add_neq(a, b).expect("first neq must succeed");
        // Constraint 1: a != c  => a: {2, 3}
        let id1 = ac.add_neq(a, c).expect("second neq must succeed");

        assert_eq!(ac.val(a), vec![2, 3]);

        // Retract first inequality
        ac.retract(id0).expect("retract must succeed");

        // CRITICAL: Value '1' in 'a' was suppressed by id0.
        // Even after retracting id0, '1' should stay suppressed because id1 (a != c) still forbids it.
        assert_eq!(ac.val(a), vec![2, 3], "Value 1 should still be suppressed by the other inequality");

        ac.retract(id1).expect("retract must succeed");
        assert_eq!(ac.val(a), vec![1, 2, 3], "All values should be restored now");
    }

    #[test]
    fn test_diamond_chain_propagation() {
        let mut ac = Engine::new();
        let a = ac.add_var([1, 2, 3]);
        let b = ac.add_var([2, 3, 4]);
        let c = ac.add_var([2, 3, 4]);
        let d = ac.add_var([3, 4, 5]);

        // Setup chain: a == b, b == d, a == c, c == d
        ac.add_eq(a, b).expect("a==b");
        ac.add_eq(b, d).expect("b==d");
        ac.add_eq(a, c).expect("a==c");
        ac.add_eq(c, d).expect("c==d");

        assert_eq!(ac.val(a), vec![3]);
        assert_eq!(ac.val(d), vec![3]);
    }

    #[test]
    fn test_inequality_chain_reaction() {
        let mut ac = Engine::new();
        // A chain where narrowing one forces another via inequalities
        let a = ac.add_var([1]);
        let b = ac.add_var([1, 2]);
        let c = ac.add_var([2, 3]);

        ac.add_neq(a, b).expect("a!=b"); // forces b to {2}
        ac.add_neq(b, c).expect("b!=c"); // forces c to {3}

        assert_eq!(ac.val(b), vec![2]);
        assert_eq!(ac.val(c), vec![3]);
    }

    #[test]
    fn test_set_constraint_and_retraction() {
        let mut ac = Engine::new();
        let a = ac.add_var([1, 2, 3]);

        let set_id = ac.set(a, 2).expect("set must succeed");
        assert_eq!(ac.val(a), vec![2]);

        ac.retract(set_id).expect("retract must succeed");
        assert_eq!(ac.val(a), vec![1, 2, 3]);
    }

    #[test]
    fn test_forbid_constraint_and_retraction() {
        let mut ac = Engine::new();
        let a = ac.add_var([1, 2, 3]);

        let forbid_id = ac.forbid(a, 2).expect("forbid must succeed");
        assert_eq!(ac.val(a), vec![1, 3]);

        ac.retract(forbid_id).expect("retract must succeed");
        assert_eq!(ac.val(a), vec![1, 2, 3]);
    }

    #[test]
    fn test_set_with_binary_interaction_and_retraction() {
        let mut ac = Engine::new();
        let a = ac.add_var([1, 2, 3]);
        let b = ac.add_var([2, 3]);

        let eq_id = ac.add_eq(a, b).expect("equality must succeed");
        let set_id = ac.set(a, 2).expect("set must succeed");

        // set(a,2) should propagate through equality to b
        assert_eq!(ac.val(a), vec![2]);
        assert_eq!(ac.val(b), vec![2]);

        // Retracting only set should keep binary propagation active.
        // Since eq(a, b) is still active and b is {2}, a remains {2}.
        ac.retract(set_id).expect("retract set must succeed");
        assert_eq!(ac.val(a), vec![2]);
        assert_eq!(ac.val(b), vec![2]);

        ac.retract(eq_id).expect("retract eq must succeed");
        assert_eq!(ac.val(a), vec![1, 2, 3]);
        assert_eq!(ac.val(b), vec![2, 3]);
    }

    #[test]
    fn test_assert_batch_equivalence() {
        // assert_batch should produce same results as sequential assert calls
        let mut ac1 = Engine::new();
        let a1 = ac1.add_var([1, 2, 3]);
        let b1 = ac1.add_var([2, 3, 4]);
        let c1 = ac1.add_var([3, 4, 5]);

        let id0 = ac1.new_constraint(Constraint::Equality(a1, b1));
        let id1 = ac1.new_constraint(Constraint::Equality(b1, c1));
        let id2 = ac1.new_constraint(Constraint::Set(a1, 3));

        ac1.assert_batch(&[id0, id1, id2]).expect("batch assert must succeed");

        // Separate sequential calls
        let mut ac2 = Engine::new();
        let a2 = ac2.add_var([1, 2, 3]);
        let b2 = ac2.add_var([2, 3, 4]);
        let c2 = ac2.add_var([3, 4, 5]);

        let id0 = ac2.new_constraint(Constraint::Equality(a2, b2));
        let id1 = ac2.new_constraint(Constraint::Equality(b2, c2));
        let id2 = ac2.new_constraint(Constraint::Set(a2, 3));

        ac2.assert(id0).expect("assert 0");
        ac2.assert(id1).expect("assert 1");
        ac2.assert(id2).expect("assert 2");

        // Results must match
        assert_eq!(ac1.val(a1), ac2.val(a2));
        assert_eq!(ac1.val(b1), ac2.val(b2));
        assert_eq!(ac1.val(c1), ac2.val(c2));
    }

    #[test]
    fn test_retract_batch_equivalence() {
        // Create two identical engines with constraints
        let mut ac1 = Engine::new();
        let a1 = ac1.add_var([1, 2, 3]);
        let b1 = ac1.add_var([1, 2, 3]);
        let c1 = ac1.add_var([1, 2, 3]);

        let id0 = ac1.add_eq(a1, b1).expect("eq 0");
        let id1 = ac1.add_neq(b1, c1).expect("neq 1");
        let id2 = ac1.set(a1, 2).expect("set 2");

        let mut ac2 = Engine::new();
        let a2 = ac2.add_var([1, 2, 3]);
        let b2 = ac2.add_var([1, 2, 3]);
        let c2 = ac2.add_var([1, 2, 3]);

        let id0_2 = ac2.add_eq(a2, b2).expect("eq 0");
        let id1_2 = ac2.add_neq(b2, c2).expect("neq 1");
        let id2_2 = ac2.set(a2, 2).expect("set 2");

        // Batch retract
        ac1.retract_batch(&[id0, id1, id2]).expect("batch retract");

        // Sequential retract
        ac2.retract(id0_2).expect("retract 0");
        ac2.retract(id1_2).expect("retract 1");
        ac2.retract(id2_2).expect("retract 2");

        // Results must match
        assert_eq!(ac1.val(a1), ac2.val(a2));
        assert_eq!(ac1.val(b1), ac2.val(b2));
        assert_eq!(ac1.val(c1), ac2.val(c2));
    }

    #[test]
    fn test_listener_notification() {
        use std::sync::{Arc, Mutex};

        let mut ac = Engine::new();
        let a = ac.add_var([1, 2, 3]);
        let b = ac.add_var([2, 3, 4]);

        // Track notifications
        let notified_vars = Arc::new(Mutex::new(Vec::new()));
        let notified_vars_clone = notified_vars.clone();

        ac.set_listener(a, move |var_id| {
            notified_vars_clone.lock().unwrap().push(var_id);
        });

        ac.set_listener(b, {
            let notified_vars_clone = notified_vars.clone();
            move |var_id| {
                notified_vars_clone.lock().unwrap().push(var_id);
            }
        });

        // Apply constraint that changes domains
        ac.add_eq(a, b).expect("equality must succeed");

        let notified = notified_vars.lock().unwrap();
        // Both a and b should have been notified during propagation
        assert!(notified.contains(&a), "Variable a should have been notified");
        assert!(notified.contains(&b), "Variable b should have been notified");
    }

    #[test]
    fn test_three_way_inequality_conflict_explanation() {
        // Three variables with domain {1, 2}, all mutually different.
        // The system is globally unsatisfiable (pigeonhole), but AC-3 cannot
        // detect this until a domain is narrowed externally.
        let mut ac = Engine::new();
        let x = ac.add_var([1, 2]);
        let y = ac.add_var([1, 2]);
        let z = ac.add_var([1, 2]);

        // AC-3 accepts all three inequality constraints without conflict.
        let neq_xy = ac.add_neq(x, y).expect("x!=y must be arc-consistent");
        let neq_xz = ac.add_neq(x, z).expect("x!=z must be arc-consistent");
        let neq_yz = ac.add_neq(y, z).expect("y!=z must be arc-consistent");

        // Assigning x=1 triggers propagation and exposes the contradiction.
        let set_x = ac.new_constraint(Constraint::Set(x, 1));
        let err = ac.assert(set_x).expect_err("conflict must be detected after set(x,1)");

        match err {
            PropagationError::DomainWipeout { var: _, ref explanation } => {
                assert!(explanation.contains(&neq_xy), "explanation must include x!=y");
                assert!(explanation.contains(&neq_xz), "explanation must include x!=z");
                assert!(explanation.contains(&neq_yz), "explanation must include y!=z");
            }
            other => panic!("expected DomainWipeout, got {:?}", other),
        }
    }
}
