# Toy Model of Physical Seshatic Identity
A simple toy model for physical Seshatic identity that satisfies the holographic principle

![clock](./images/clock.png)

The above screenshot is from a simple clock ticking while moving through the maze.

- Yellow: `Player` (current position of the player)
- Green: `Follower` (past positions of the player)
- Empty: `Unknown` (indeterminate value)
- Red: `0` (observable)
- Blue: `1` (observable)

To install the demo:

```text
cargo cargo install --example advancedresearch-toy_model_of_physical_seshatic_identity holomaze
```

To run the demo:

```text
holomaze
```

To use this library in a Rust project, add it to Cargo toml and import by the following:

```text
use holomaze::*;
```

### Useful links

- [List of Open Problems](https://github.com/advancedresearch/toy_model_of_physical_seshatic_identity/issues/16)

### The basics of this toy model

Physical Seshatic identity is modeled by designing a mathematical language bias such that a player (represented as `Player`)
can only navigate through some space by a unique path defined by "laws of nature".

This space in the toy model is a 4x4 matrix to enable efficient enumeration of all possible worlds (using the `u16` type in Rust).
While the player can have multiple choices, only one choice can lead to eventual an exit or "goal":

```text
s ? ? ?    s = start
? ? ? ?    g = goal
? ? ? ?
? ? ? g
```

Unlike most maze puzzles, this puzzle evaluates "laws of nature" over all possible worlds,
and the player is only allowed to move such that the bit at the player's old position is equal
to the bit where the player wants to move.
This must hold in every possible world, in order to provide a valid move.
The "laws of nature" have no epistemological access to the player state,
but treats each possible world as 4x4 bits where each cell is either 0 or 1.

Since there are 4x4 bits, one gets `2^(4*4) = 65536` worlds to check before moving the player.
So, the "laws of nature" are evaluated 65536 times by inserting bits into indeterminate cell values.
This is done twice, one to check for valid player moves among possible worlds,
two to perform a Dr. Strange check that no impossible world can result in the new world.
The Dr. Strange check makes sure that there exists some reversible algorithm to the "laws of nature".

There are 3 kinds of indeterminate cell values:

- `Player`: Used to track movement of the player
- `Follower`: Used to label the past of the player for easy visualization
- `Unknown`: Indeterminate cell value that is neither player's position nor the player's past

The basic idea is that the upper left corner can be treated as an input source of information.
When the player starts the game, it can be thought of as entering the maze.
As time goes by, when the player moves around in the maze,
new bits of information arrives into the maze, that can influence the player's ability to reach the goal.
However, there is no way to tell what kind of bits will arrive into the maze as time goes by.
Therefore, the game is set up such that all possible bits are checked,
meaning that the player can move only when the move is possible regardless of the state of the bits.
This allows the player state to be indeterminate, that is never any specific configuration of bits.
The player is all possible states that the player can be.

Since new bits arrive into the maze after the player,
the player can not block the entry to the maze.
If this is allowed, then in many puzzles the player can cheat by moving time forward while standing still in the top left corner.
The cheat is not allowed because it would provide multiple paths to the goal.
There are some cases where a player can stand still in the middle of the maze,
e.g. when there is a conditional move, but this is either intentional by the designer of the puzzle,
either to find the correct path or to mislead the player,
or it can be an unintentional dead end that was not caught under testing the puzzle.

When reaching the goal, one has tracked the metaphysical identity of the dynamic bit that represents the player through the maze.
A finished game is a proof of the unique path through the maze that makes the bit at the goal position
equal to the bit at the start position, but applying "laws of nature" some finite number of times.

There are two sides to the gameplay: People who solve puzzles and people who design puzzles.
They both have to follow the rules, but there is no need to check manually that the rules are followed.
If the rules are broken, the intended puzzle design will simply not work.
The game is set up such that there is no way to cheat, for neither "laws of nature", nor the player.
One can build different mazes by swapping out "laws of nature",
which represents puzzle challenges to the player.

It is possible to construct puzzles that have no solution,
but if there exists some solution, then there is only one solution, a unique path.
This semantics of a unique path in game design is unusual,
because the number of paths usually explodes combinatorially.
Yet, this setup provides a guarantee of at most one path as solution,
by leveraging mathematical language bias design.
While all possible paths are not evaluated directly,
it follows from the way "laws of nature" are evaluated across possible worlds,
plus the requirement that the player can not block followers from entering the maze.

The Dr. Strange check is simply to help the puzzle designer discover bugs
that might happen under other possible world that the ones being tested.
To simulate physical Seshatic identity, you want the property that time direction can be reversed in principle.

When there are no observables, the "laws of nature" has no available memory and the player is not able to move in circles.
So, analytically, this is the simplest case to reason about.
The "laws of nature" (even without observables) might allow teleportation, so there are `(4*4)! = 20 922 789 888 000`,
almost 21 trillion possible paths!
This is would be very expensive to check one by one with today's technology.
So, the only practical way to design this toy model is by leveraging mathematical language bias design.

### Observables and the holographic principle

When "laws of nature" sets a bit to a specific value in every possible world,
this bit becomes an observable that is used to filter out possible worlds in future moves.

However, it follows logically that if a deterministic algorithm outputs a specific bit in every possible world,
then the same algorithm can not change its mind later.
The collapse of an indeterminate value into an observable can only happen when this already happened in all possible futures or pasts.
It is kind of like having a time police that prevents inconsistency across the timeline.

Yet, there is an incredible trick that allows dynamics in observables.
The "laws of nature" can depend on the state of observables to determine the value of new observables.
Since observables are used to filter out possible worlds,
one can avoid the possible worlds that would force the observables to stay the same values in the same locations.
This trick results in possible dynamics, but only if there exists some observables prior to the start of the game.
With other words, one can design a map with prior observables which information gets preserved,
although the shape of that information might change over time.

For example, when designing a clock, one allocates prior observables and makes "laws of nature" toggle these bits using e.g. binary counting.

The trick used to give dynamics in observables is the analogue of the holographic principle in theoretical physics.

All information within the space that might be used by "laws of nature" is preserved and can neither be created nor destroyed by it
(technically, only the information in the space, after "laws of nature" has been applied at least once, counts).
This is ensured by counting the observables before and after moves.
The move becomes invalid if information is created or destroyed.

There are some things the player can do that are not depending on this information,
but in that case these things are possible to do in every possible world.
Information encoded as observables allows greater variety in how the player can move,
which essentially brings the number of paths from 21 trillion to ... well... a very huge number (how to calculate this is an open problem).
So, it is possible to design puzzles where the player gets stuck in the maze, for all eternity.
There might also be just one such path, that the player only survives for eternity when making one correct choice at each moment in time.

With other words, all the memory that might be used by "laws of nature" must be given beforehand.
In theoretical physics, this corresponds to the number of Planck areas that cover some region of space-time.
It does not mean that the location is actually located physically at the boundary.
Instead, it means the preservation of this area as information over time.
As long the "laws of nature" never change, it can neither create nor destroy this information.
The information might be transformed, e.g. a bit `0` might be turned into `1` and put somewhere else,
but it can not vanish.
There is always some way to reconstruct the past: Both the player moves in a unique path
and the information used as memory is preserved, so there exists some algorithm that can reverse time direction.

However, only because some algorithm with reversed time direction exists,
does not imply that this algorithm is obtainable by studying the "laws of nature" as a black box.
This is because eternal paths might exist that are undecidably indistinguishable from other possible "laws of nature".
When there is an infinite amount of memory possible, determining the reverse algorithm gets undecidable.
Yet, since there is only a finite amount of memory possible in this toy model,
it is currently unknown whether one can determine the reverse algorithm using another deterministic algorithm.
To prove this or finding a counter-proof is an open problem.

### About transcendence and resurrection from the dead

This toy model does not support transcendence, in the typical way that people play the game.

Transcendence, or resurrection from the dead,
is when the bit of the player is temporarily split into separate parts and put together again at a later moment.
Currently, it is an open problem whether one can make multiple moves and recover the player,
after confirming that the player is dead at some intermediate step.
The first person who finds a such proof of transcendence in this toy model,
will receive the title "Neo" as an reward (Neo from the movie "The Matrix").
The first person who finds a counter-proof, will receive the corresponding title "Agent Smith".

Alternatively, if it is not possible to achieve transcendence,
then it might be possible to modify this toy model to support it.
People who find such algorithms, will receive the corresponding title "Architect".

So, this toy model is not just a puzzle in itself, you can also play the meta-game:

- Neo: Finds a set of "laws of nature" that allows transcendence
- Agent Smith: Shows that there is no "laws of nature" that allows transcendence
- Architect: Modifies the toy model to support transcendence (there might be many ways to do it)

If a Neo shares the solution, then it is very likely that one can modify it to find many other solutions of transcendence.
So, please "save us", Neo. Lead us to salvation! 😜

### Short introduction to the holographic principle

The holographic principle in theoretical physics states that the information of some region of space-time
is constrained to the area in Planck units at the boundary of that region.

While this principle might seem mysterious at first, it is a natural property of laws of nature
being evaluated over all possible worlds to give observers metaphysical Seshatic identity.

This toy model is used to explain why there is a holographic principle in nature.
It is designed in a way to let people test out arbitrary "laws of nature" with respect to the toy model,
but with mathematical language design bias using the latest research from fundamental Path Semantics.
With other words, you can play around with the toy model, but you can not cheat in a way
that violates the holographic principle. Therefore, the holographic principle holds across the multiverse.

### Short introduction to Seshatism vs Platonism

Seshatism is a language bias dual to Platonism in Path Semantics.
These terms are applied across many domains, but often hold specific meaning within each domain that can be identified.
By building intuition from experience, people can learn to recognize the sides of this duality in new domains they have no previous experience with.

The duality Seshatism vs Platonism is designed to be used as fundamental bases in Joker Calculus,
and to explore the logical properties of the core axiom in Path Semantics and its variants.

Seshat is an Ancient Egyptian goddess of the library and daughter of Ma'at (who represents justice or balance).
In Ancient Greek terminology she might be thought of as a Muse (the deities that inspired Mouseion,
the institution that built the library of Alexandria),
but Seshat pre-dates the idea of Muses so much that she must be treated as lost in historical obscurity at that time.

When Plato wrote about Thoth (which is likely a later deity than Seshat that took over some of her roles),
he did not mention Seshat at all, which could signify that this knowledge was lost when Plato writes.
Yet, this unbalanced representation of the mythology of the invention of writing, violated the principle of Ma'at.
Hence, Plato unleashed a "Pharaoh's curse" on humanity that might have set humanity on a path of self-destruction.

Seshatism is designed to represent the philosophy of restoration and reconstruction of history,
by crediting knowledge by causality, the dual way of crediting knowledge by abstraction (Platonism).

### Physical Seshatic identity

In fundamental Path Semantics, the time interpretation of the core axiom explains how to move propositional content from
one moment in time to the next moment. This can not happen by some tautology in normal logic,
so one has to study something like the core axiom to understand the logical properties of time.
What makes the core axiom special is that it requires a "quality" operator.

The research in fundamental Path Semantics has concluded that there exists a classical model of quality (`~~`),
that is a partial equivalence relation (think "equality" without the "e" that represents reflection),
by introducing a "qubit" operator `~` (it is tautological congruent `(a == b)^true => (~a == ~b)` with the HOOO EP axioms):

```text
(a ~~ b) == (~a & ~b & (a == b))
```

So, everything in this model can explained using only the qubit operator `~`, which extends normal logic to infinite-valued logic.
Quality can simply be defined using the qubit operator and we get a more general model that looks more like quantum physics.
It is not an actual quantum mechanical qubit operator, but it is similar enough to apply ideas from this research to the philosophy of physics.

The qubit operator is modeled by using the input bit vector as a seed for a pseudorandom bit vector,
usually with the Sesh property that `!~a == ~!a`.
When both the Sesh axiom and Excluded Middle is used in constructive logic, this model represents sequences of bits.
In theology, one has to relax either the Sesh axiom or Excluded Middle to model Agnostic language bias
in Path Semantical sense (not how it was defined by Huxley).

To model Seshatism relative to the core axiom of Path Semantics,
one can define it as:

```test
!~a
```

Which is the same as `!(a ~~ a)`, a negation of reflexivity.
This is only possible to express using a partial equivalence,
which explains why partial equivalence is so important for logical temporality.

Negated reflexivity (some Hegelians might be familiar with the idea) has the property that no other proposition can be qual to it (think "equal" without the "e"):

```text
!(a ~~ a) => !(a ~~ b)
```

Hence, Seshatism here models originality of logical propositions.
It fits the intuition of metaphysical identity that a person exists in only one approximate location in space-time
and can not be teleported somewhere else by copying the bits of information about the person (the no-cloning theorem in quantum physics)
while preserving the original person.
Therefore, it is called a "physical Seshatic identity".

However, physical Seshatic identity is merely a play with a logical illusion,
due to being biased toward Platonism when selecting a core axiom of Path Semantics
where `a ~~ a` is a non-tautological identity of the salvation/nirvana/enlightenment analogue in theology.
This is an internal difference in philosophy, which means that the distinction can only be
told internally within the same language.
From the outside, `~a` and `!~a` can be swapped without being able to tell the difference (a flipped random bit vector looks random).

With other words, one has to be careful to not view these ideas as ontological absolutes,
but acknowledge that these are just language biases in the sense of the version of Wittgensteineanism applied in research of Path Semantics.
From experience of using Joker Calculus to classify language biases, it is currently believed that
humans have evolved internalization of Joker Calculus to solve social problems,
alternating between authenticity and inauthenticity in sense of Heidegger,
in a way that is believed to biologically driven and not predicated on fundamental ontology of reality.
This can be tested scientificially by observing changes in thought patterns resulting from stimuli biased by expressions in Joker Calculus.

So, the Plato's cave metaphor applies inside mathematics as well.
The divide between inside and outside the cave results in Inside and Outside theories of mathematics,
where most mathematical theory as practiced today is Inside theory and e.g. the theory of Avatar Extensions is an Outside theory.
