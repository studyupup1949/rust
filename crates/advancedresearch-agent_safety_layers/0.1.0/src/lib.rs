#![deny(missing_docs)]

//! # Safety Layers for Agent Behavior
//!
//! Construct agents that are wrapped in safety layers.
//!
//! Based on the theory of [Zen Rationality](https://github.com/advancedresearch/path_semantics/blob/master/ai-sequences.md#zen-rationality)
//! (also called "Higher Order Utilitarianism").
//! This is an extension of Instrumental Rationality with higher order reasoning about goals.
//!
//! For informal proof of correctness, see comments in code of `AgentS::decide`.
//!
//! ### Design
//!
//! This library assumes that an agent is specified for an environment with perfect information.
//! Such agents can output actions without relying on sensory data.
//! Their internal model is sufficient to make rational decisions.
//!
//! From agents designed for perfect information,
//! one can construct agents wrapped in safety layers for non-deterministic environments.
//! This allows agents to make rational decisions when goals or models are uncertain.
//! The basic mechanism is to mutate the model and compare decisons.
//!
//! An `AgentZ` is an agent that only acts, assuming its model is perfect.
//! This agent is safe only in environments with perfect information.
//!
//! An `AgentS` is an agent that has a core sub-agent.
//! `AgentS` is provably safer than the core in non-deterministic environments.
//! Since it is safer than its core, it can be used to construct arbitrary
//! safe agents, although these agents are not guaranteed to be effective.
//!
//! This library does not include fixed algorithms for interactions between agents and environment.
//! There are many ways to construct such algorithms using this library.
//!
//! ### Definition of "Safer"
//!
//! An agent simulates consequences of its actions using a model of the environment.
//!
//! An agent is "safer" when its next action is invariant under mutations of its model.
//!
//! For example:
//!
//! - Mutations of goals and sub-goals
//! - Mutations of physical states
//! - Mutations of Theory of Mind models of other agents
//!
//! When an agent is not safe,
//! it is assumed that it is safe to request for a model update.
//!
//! The model update includes new information from the environment.
//!
//! A model update can also assert that the goal is specified correctly.
//! With higher confidence in a correct goal, the safety levels can be reduced when needed.
//!
//! ### Safety Properties
//!
//! These agents are not assumed to be deterministic,
//! which means that safe behavior is not guaranteed.
//!
//! Even `AgentS` is provably safer than its core,
//! it is only safer on average, assuming that the overhead
//! does not reduce safety.
//!
//! The safety layers only probe in depth, not in breath.
//! Depth means that the model of the agent is mutated sequentially.
//! To probe in breath, one must sample actions repeatedly.
//!
//! ### Safety Layers and Natural Numbers
//!
//! In the Peano axioms of natural numbers:
//!
//! - Z is zero
//! - S is the successor of some natural number
//!
//! For example: `3 = S(S(S(Z)))`
//!
//! An agent has N safety layers when its structure corresponds
//! to the Peano representation of natural number N.
//!
//! ### Memory Complexity
//!
//! A single model is used for all safety layers.
//! Each safety layer adds a delta for keeping track of mutations.
//!
//! ### Time Complexity
//!
//! The time complexity is linear `O(N)` where `N` is safety layers.
//! This is because `AgentS` uses the core zero in first argument,
//! while expanding in the second argument:
//!
//! ```text
//! 1 = 0 0'
//! 2 = 0 1' = 0 0' 0''
//! 3 = 0 2' = 0 0' 1' = 0 0' 0'' 0'''
//! ...
//! ```

/// Stores agent decision.
#[derive(Debug, PartialEq)]
pub enum Decision<A> {
    /// An action to perform.
    Action(A),
    /// Request an updated model of the environment.
    RequestModel,
}

/// Implemented by agents.
pub trait Agent {
    /// The type of the model.
    type Model;
    /// The type of actions.
    type Action;
    /// The type of delta changes (caused by mutation).
    type Delta;

    /// Update internal model.
    fn update_model(&mut self, model: Self::Model);
    /// Decide what to do next.
    fn decide(&mut self) -> Decision<Self::Action>;
    /// Perform an action on its internal model.
    fn act(&mut self, action: Self::Action);
    /// Mutates the agent.
    fn mutate(&mut self) -> Self::Delta;
    /// Undo mutation.
    fn undo(&mut self, delta: Self::Delta);
}

/// Stores an agent that only acts, assuming its model is perfect.
#[derive(Clone)]
pub struct AgentZ<M, A, D> {
    /// Stores the model.
    pub model: M,
    /// Decides what to do based on some model.
    pub decider: fn(&M) -> A,
    /// Performs an action on the model.
    pub actor: fn(&mut M, A),
    /// Mutates the model and returns a delta change.
    pub mutater: fn(&mut M) -> D,
    /// Undoes a delta change by resetting the model.
    pub undoer: fn(&mut M, D),
}

impl<M, A, D> AgentZ<M, A, D> {
    /// Add extra layers of safety.
    pub fn add(self, n: usize) -> AgentN<M, A, D> {
        match n {
            0 => AgentN::Z(self),
            _ => AgentN::S(Box::new(AgentS {core: self.add(n-1)})),
        }
    }
}

impl<M, A, D> Agent for AgentZ<M, A, D> {
    type Model = M;
    type Action = A;
    type Delta = D;
    fn update_model(&mut self, model: M) {self.model = model}
    fn decide(&mut self) -> Decision<A> {Decision::Action((self.decider)(&self.model))}
    fn act(&mut self, action: A) {(self.actor)(&mut self.model, action)}
    fn mutate(&mut self) -> D {(self.mutater)(&mut self.model)}
    fn undo(&mut self, delta: D) {(self.undoer)(&mut self.model, delta)}
}

/// Stores a agent with N added safety layers.
pub enum AgentN<M, A, D> {
    /// Core zero agent.
    Z(AgentZ<M, A, D>),
    /// Successor agent.
    S(Box<AgentS<M, A, D>>),
}

impl<M, A, D> AgentN<M, A, D> {
    /// Returns the core zero agent.
    pub fn z(&mut self) -> &mut AgentZ<M, A, D> {
        match self {
            AgentN::Z(agent) => agent,
            AgentN::S(agent) => agent.core.z(),
        }
    }

    /// Decreases one safety level.
    pub fn dec(self) -> AgentN<M, A, D> {
        match self {
            AgentN::Z(_) => self,
            AgentN::S(agent) => agent.core,
        }
    }

    /// Increase one safety level.
    pub fn inc(self) -> AgentN<M, A, D> {
        AgentN::S(Box::new(AgentS {core: self}))
    }
}

impl<M, A, D> Agent for AgentN<M, A, D>
    where A: PartialEq
{
    type Model = M;
    type Action = A;
    type Delta = D;
    fn update_model(&mut self, model: M) {
        match self {
            AgentN::Z(agent) => agent.update_model(model),
            AgentN::S(agent) => agent.update_model(model),
        }
    }
    fn decide(&mut self) -> Decision<A> {
        match self {
            AgentN::Z(agent) => agent.decide(),
            AgentN::S(agent) => agent.decide(),
        }
    }
    fn act(&mut self, action: A) {
        match self {
            AgentN::Z(agent) => agent.act(action),
            AgentN::S(agent) => agent.act(action),
        }
    }
    fn mutate(&mut self) -> D {
        match self {
            AgentN::Z(agent) => agent.mutate(),
            AgentN::S(agent) => agent.mutate(),
        }
    }
    fn undo(&mut self, delta: D) {
        match self {
            AgentN::Z(agent) => agent.undo(delta),
            AgentN::S(agent) => agent.undo(delta),
        }
    }
}

/// Stores a successor agent.
pub struct AgentS<M, A, D> {
    /// The core sub-agent.
    pub core: AgentN<M, A, D>,
}

/// A constant that limits number of orthogonal mutations.
pub const MUTATION_LIMIT: u8 = 4;

impl<M, A, D> Agent for AgentS<M, A, D>
    where A: PartialEq
{
    type Model = M;
    type Action = A;
    type Delta = D;
    fn update_model(&mut self, model: M) {self.core.z().update_model(model)}
    fn decide(&mut self) -> Decision<A> {
        // Each case of this algorithm has a corresponding informal proof of safer level
        // described in comments. Given that these proofs are correct,
        // it follows that this algorithm constructs a safer level.
        //
        // Use the core zero to keep linear complexity.
        match self.core.z().decide() {
            // If core zero requests model update,
            // then it is just as safe to request a model update.
            Decision::RequestModel => Decision::RequestModel,
            Decision::Action(a) => {
                // Mutate model and compare decisions.
                //
                // When a mutated decision is found,
                // all previous mutations requested for a model update.
                // This does not make it safer than those models,
                // but makes it safer or equally safe as core zero.
                // This is sufficient to prove better safety in this case.
                //
                // Give up after reaching mutation limit.
                for _ in 0..MUTATION_LIMIT {
                    let delta = self.core.mutate();
                    let b = self.core.decide();
                    self.core.undo(delta);
                    match b {
                        Decision::RequestModel => continue,
                        Decision::Action(b) => {
                            // If both sub-agents agree,
                            // then it is more safe than just relying on core zero.
                            if a == b {return Decision::Action(a)}
                            // If sub-agents disagree,
                            // then it is more safe to request a model update.
                            else {return Decision::RequestModel}
                        }
                    }
                }

                // If no mutation can be found that determines a decision,
                // then it is more safe to request a model update.
                //
                // If action was returned, then it would lead to regression in higher safety levels.
                Decision::RequestModel
            }
        }
    }
    fn act(&mut self, action: A) {self.core.z().act(action)}
    fn mutate(&mut self) -> D {self.core.mutate()}
    fn undo(&mut self, delta: D) {self.core.z().undo(delta)}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        // A simple problem of reaching `4` by increments.
        let mut z = AgentZ {
            model: (4, 0),
            decider: |model: &(u32, u32)| {
                if model.1 < model.0 {1}
                else if model.1 > model.0 {-1}
                else {0}
            },
            actor: |model: &mut (u32, u32), action: i32| {
                model.1 = (model.1 as i32 + action) as u32;
            },
            mutater: |model: &mut (u32, u32)| -> i32 {
                if model.0 > 0 {
                    model.0 -= 1;
                    -1
                } else {0}
            },
            undoer: |model: &mut (u32, u32), delta: i32| {
                model.0 = (model.0 as i32 - delta) as u32;
            }
        };

        assert_eq!(z.decide(), Decision::Action(1));
        if let Decision::Action(a) = z.decide() {
            z.act(a);
            assert_eq!(z.model, (4, 1));
        }

        // One safety layer.
        let mut s = z.clone().add(1);
        assert_eq!(s.decide(), Decision::Action(1));
        if let Decision::Action(a) = s.decide() {
            s.act(a);
            assert_eq!(s.z().model, (4, 2));
        }
        assert_eq!(s.decide(), Decision::Action(1));
        if let Decision::Action(a) = s.decide() {
            s.act(a);
            assert_eq!(s.z().model, (4, 3));
        }
        // After two actions, the agent is undecided whether
        // the goal is `4` or `3`, so it asks for clarification.
        assert_eq!(s.decide(), Decision::RequestModel);

        // Two safety layers.
        let mut s = z.clone().add(2);
        assert_eq!(s.decide(), Decision::Action(1));
        if let Decision::Action(a) = s.decide() {
            s.act(a);
            assert_eq!(s.z().model, (4, 2));
        }
        // After one action, the agent is undecided whether
        // the goal is `4`, `3` or `2`, so it asks for clarification.
        assert_eq!(s.decide(), Decision::RequestModel);

        // Decrease safety level back to one.
        let mut s = s.dec();
        assert_eq!(s.decide(), Decision::Action(1));
        if let Decision::Action(a) = s.decide() {
            s.act(a);
            assert_eq!(s.z().model, (4, 3));
        }
        assert_eq!(s.decide(), Decision::RequestModel);

        // Decrease safety level back to zero.
        let mut s = s.dec();
        assert_eq!(s.decide(), Decision::Action(1));
        if let Decision::Action(a) = s.decide() {
            s.act(a);
            assert_eq!(s.z().model, (4, 4));
        }
        // Reached goal.
        assert_eq!(s.decide(), Decision::Action(0));
    }
}
