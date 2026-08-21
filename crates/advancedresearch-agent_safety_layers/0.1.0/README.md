# Safety Layers for Agent Behavior

Construct agents that are wrapped in safety layers.

Based on the theory of [Zen Rationality](https://github.com/advancedresearch/path_semantics/blob/master/ai-sequences.md#zen-rationality)
(also called "Higher Order Utilitarianism").
This is an extension of Instrumental Rationality with higher order reasoning about goals.

For informal proof of correctness, see comments in code of `AgentS::decide`.

### Design

This library assumes that an agent is specified for an environment with perfect information.
Such agents can output actions without relying on sensory data.
Their internal model is sufficient to make rational decisions.

From agents designed for perfect information,
one can construct agents wrapped in safety layers for non-deterministic environments.
This allows agents to make rational decisions when goals or models are uncertain.
The basic mechanism is to mutate the model and compare decisons.

An `AgentZ` is an agent that only acts, assuming its model is perfect.
This agent is safe only in environments with perfect information.

An `AgentS` is an agent that has a core sub-agent.
`AgentS` is provably safer than the core in non-deterministic environments.
Since it is safer than its core, it can be used to construct arbitrary
safe agents, although these agents are not guaranteed to be effective.

This library does not include fixed algorithms for interactions between agents and environment.
There are many ways to construct such algorithms using this library.

### Definition of "Safer"

An agent simulates consequences of its actions using a model of the environment.

An agent is "safer" when its next action is invariant under mutations of its model.

For example:

- Mutations of goals and sub-goals
- Mutations of physical states
- Mutations of Theory of Mind models of other agents

When an agent is not safe,
it is assumed that it is safe to request for a model update.

The model update includes new information from the environment.

A model update can also assert that the goal is specified correctly.
With higher confidence in a correct goal, the safety levels can be reduced when needed.

### Safety Properties

These agents are not assumed to be deterministic,
which means that safe behavior is not guaranteed.

Even `AgentS` is provably safer than its core,
it is only safer on average, assuming that the overhead
does not reduce safety.

The safety layers only probe in depth, not in breath.
Depth means that the model of the agent is mutated sequentially.
To probe in breath, one must sample actions repeatedly.

### Safety Layers and Natural Numbers

In the Peano axioms of natural numbers:

- Z is zero
- S is the successor of some natural number

For example: `3 = S(S(S(Z)))`

An agent has N safety layers when its structure corresponds
to the Peano representation of natural number N.

### Memory Complexity

A single model is used for all safety layers.
Each safety layer adds a delta for keeping track of mutations.

### Time Complexity

The time complexity is linear `O(N)` where `N` is safety layers.
This is because `AgentS` uses the core zero in first argument,
while expanding in the second argument:

```text
1 = 0 0'
2 = 0 1' = 0 0' 0''
3 = 0 2' = 0 0' 1' = 0 0' 0'' 0'''
...
```
