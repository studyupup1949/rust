# Advanced Architecture (ADAR)

[![Crates.io](https://img.shields.io/crates/v/adar.svg)](https://crates.io/crates/adar)
[![Downloads](https://img.shields.io/crates/d/adar.svg)](https://crates.io/crates/adar)
[![Docs](https://docs.rs/adar/badge.svg)](https://docs.rs/adar/latest/adar/)

Adar is a collection of architectural tools that help you write more readable and performant code.

## Flags

[Flags](`crate::enums::Flags`) is a type-safe and verbose bitwise flag container. Most of the flag operations are inlined.

### Features

- Union, Intersect
- Serialization (requires `serde` feature)
- Conversion to and from raw values
- Intuitive syntax

### Example

```rust
use adar::prelude::*;

#[FlagEnum]
enum MyFlag {F1, F2, F3}

let mut a = Flags::from(MyFlag::F1);
a.set(MyFlag::F2);
let mut b = MyFlag::F1 | MyFlag::F2 | MyFlag::F3;
b.reset(MyFlag::F1);

println!("a: {:?}, {:03b}", a, a.into_raw());
println!("b: {:?}, {:03b}", b, b.into_raw());

println!("a.any(F1,F3): {:?}", a.any(MyFlag::F1 | MyFlag::F3));
println!("a.all(F1,F3): {:?}", a.all(MyFlag::F1 | MyFlag::F3));

println!("a.intersect(b): {:?}", a.intersect(b));
println!("a.union(b): {:?}", a.union(b));
println!("full(): {:?}", Flags::<MyFlag>::full());
```

<details>
<summary>Click to see the output</summary>
<code><b>>> cargo run --example flags</b></code>

```ignore
a: (F1,F2), 011
b: (F2,F3), 110
a.any(F1,F3): true
a.all(F1,F3): false
a.intersect(b): (F2)
a.union(b): (F1,F2,F3)
full(): (F1,F2,F3)
```

</details>

<details>
<summary>Click to see the generated code</summary>
<code><b>>> cargo expand --example flags</b></code>

```rust,ignore
...
#[repr(u32)]
enum MyFlag {
    F1 = 1,
    F2 = 2,
    F3 = 4,
}
...
impl std::ops::BitOr for MyFlag
where
    Self: adar::prelude::ReflectEnum,
{
    type Output = adar::prelude::Flags<Self>;
    fn bitor(self, rhs: Self) -> Self::Output {
        Flags::empty() | self | rhs
    }
}
...
```

</details>

## StateMachine

### Features

- [State](`crate::state_machine::State`) callback:
  - [on_enter](`crate::state_machine::State::on_enter`) - Called when the state machine enters a state
  - [on_update](`crate::state_machine::State::on_update`) - Called when the state machine is updated (always for the current state)
  - [on_leave](`crate::state_machine::State::on_leave`) - Called when the state machine leaves a state
- [Machine](`crate::state_machine::Machine`) callback:
  - [on_update](`crate::state_machine::Machine::on_update`) - Called when update is called
  - [on_transition](`crate::state_machine::Machine::on_transition`) - Called at each transition (after [on_leave](`crate::state_machine::State::on_leave`), before [on_enter](`crate::state_machine::State::on_enter`))
- Pass arguments to updates (see [update_args](`crate::state_machine::StateMachine::update_args`), [run_args](`crate::state_machine::StateMachine::run_args`), [transition_args](`crate::state_machine::StateMachine::transition_args`))
- Store context in the [StateMachine](`crate::state_machine::StateMachine`) (see [new_context](`crate::state_machine::StateMachine::new_context`), with up to 8 generic parameters)
- Operating modes
  - Non-blocking mode (see [update_args](crate::state_machine::StateMachine::update_args))
  - Blocking mode (see [run_args](crate::state_machine::StateMachine::run_args))
- End state (see [EndState](crate::state_machine::EndState), [is_finished](crate::state_machine::HasEndState::is_finished))
- Sync only

### Example

[![](https://mermaid.ink/img/pako:eNp1kVFPgzAUhf8Kub4ZtkiBMfpgYpxZlixithmjYpYKF0ZW6FKKOpf9d8sYTjHep3tvv3NOm-4gEjEChVIxhaOMpZLlvTcSFoauiLOyHGFiSIyNJOOcnjFCzEhwIen7KlPY4VKJWBxJwlrylbNo3SG3yLl4b027aAM_n78Yvd6lMVdi02zqjlJaX6c-GKOaIYu3BjXGwXK-CO6Wo_vZ1WIS3DZ8C2jNMfAgE1rweDOdBg9dXmiyeUQbPEnmLMF_E07I74x6_zcFTEhlFgNVskITcpQ5q0fY1W4hqBXmGALVbczkOoSw2GvNhhVPQuStTIoqXQFNGC_1VG3i09d9byUWMcprURUKqDvwDyZAd_AB1PbtvmUPLddziUM81zJhqyGr79ue49nEcayB5wzdvQmfh9iL_tBz_J-1_wJ7ja8f?type=png)](https://mermaid.live/edit#pako:eNp1kVFPgzAUhf8Kub4ZtkiBMfpgYpxZlixithmjYpYKF0ZW6FKKOpf9d8sYTjHep3tvv3NOm-4gEjEChVIxhaOMpZLlvTcSFoauiLOyHGFiSIyNJOOcnjFCzEhwIen7KlPY4VKJWBxJwlrylbNo3SG3yLl4b027aAM_n78Yvd6lMVdi02zqjlJaX6c-GKOaIYu3BjXGwXK-CO6Wo_vZ1WIS3DZ8C2jNMfAgE1rweDOdBg9dXmiyeUQbPEnmLMF_E07I74x6_zcFTEhlFgNVskITcpQ5q0fY1W4hqBXmGALVbczkOoSw2GvNhhVPQuStTIoqXQFNGC_1VG3i09d9byUWMcprURUKqDvwDyZAd_AB1PbtvmUPLddziUM81zJhqyGr79ue49nEcayB5wzdvQmfh9iL_tBz_J-1_wJ7ja8f)

```rust,no_run
use adar::prelude::*;
use std::{process::Command, time::Duration};

#[StateEnum]
#[ReflectEnum] // Optional. (Used here to print the name of the state)
enum TrafficLight {
    Go,
    GetReady,
    StopIfSafe,
    Stop,
}

impl TrafficLight {
    const YELLOW_DURATION: Duration = Duration::from_secs(1);
    const GO_STOP_DURATION: Duration = Duration::from_secs(2);
}

impl Machine for TrafficLight {
    fn on_transition(&mut self, new_state: &Self::States, _context: &mut Self::Context) {
        Command::new("clear")
            .status()
            .expect("Failed to clear screen!");

        println!("{}", new_state.name());
    }
}

impl State for Go {
    fn on_enter(&mut self, _args: Option<&mut Self::Args>, _context: &mut Self::Context) {
        println!("⚫\n⚫\n🟢");
    }
    fn on_update(
        &mut self,
        _args: Option<&mut Self::Args>,
        _context: &mut Self::Context,
    ) -> Option<Self::States> {
        std::thread::sleep(TrafficLight::GO_STOP_DURATION);
        Some(StopIfSafe.into())
    }
}

impl State for GetReady {
    fn on_enter(&mut self, _args: Option<&mut Self::Args>, _context: &mut Self::Context) {
        println!("🔴\n🟡\n⚫");
    }
    fn on_update(
        &mut self,
        _args: Option<&mut Self::Args>,
        _context: &mut Self::Context,
    ) -> Option<Self::States> {
        std::thread::sleep(TrafficLight::YELLOW_DURATION);
        Some(Go.into())
    }
}

impl State for StopIfSafe {
    fn on_enter(&mut self, _args: Option<&mut Self::Args>, _context: &mut Self::Context) {
        println!("⚫\n🟡\n⚫");
    }
    fn on_update(
        &mut self,
        _args: Option<&mut Self::Args>,
        _context: &mut Self::Context,
    ) -> Option<Self::States> {
        std::thread::sleep(TrafficLight::YELLOW_DURATION);
        Some(Stop.into())
    }
}

impl State for Stop {
    fn on_enter(&mut self, _args: Option<&mut Self::Args>, _context: &mut Self::Context) {
        println!("🔴\n⚫\n⚫")
    }
    fn on_update(
        &mut self,
        _args: Option<&mut Self::Args>,
        _context: &mut Self::Context,
    ) -> Option<Self::States> {
        std::thread::sleep(TrafficLight::GO_STOP_DURATION);
        Some(GetReady.into())
    }
}

fn main() {
    StateMachine::new(Stop).run();
}
```

<details>
<summary>Click to see the output</summary>
<code><b>>> cargo run --example statemachine_trafficlight</b></code>

```ignore
Stop
🔴
⚫
⚫
... (5s)
GetReady
🔴
🟡
⚫
... (2s)
Go
⚫
⚫
🟢
... (5s)
StopIfSafe
⚫
🟡
⚫
... (2s, then repeats)
```

</details>
<details>
<summary>Click to see the generated code</summary>
<code><b>>> cargo expand --example statemachine_trafficlight</b></code>

```rust,ignore
...
enum TrafficLight {
    Go(Go),
    GetReady(GetReady),
    StopIfSafe(StopIfSafe),
    Stop(Stop),
}
impl adar::prelude::ReflectEnum for TrafficLight {
    type Type = u32;
    fn variants() -> &'static [adar::prelude::EnumVariant<TrafficLight>] {
        const VARIANTS: &[adar::prelude::EnumVariant<TrafficLight>] = &[
            EnumVariant::new("Go", None),
            EnumVariant::new("GetReady", None),
            EnumVariant::new("StopIfSafe", None),
            EnumVariant::new("Stop", None),
        ];
        VARIANTS
    }
    fn count() -> usize {
        4usize
    }
    fn name(&self) -> &'static str {
        match self {
            Self::Go { .. } => "Go",
            Self::GetReady { .. } => "GetReady",
            Self::StopIfSafe { .. } => "StopIfSafe",
            Self::Stop { .. } => "Stop",
        }
    }
}
struct Go;
impl adar::prelude::StateTypes for Go {
    type States = TrafficLight;
    type Args = ();
    type Context = ();
}
impl Into<TrafficLight> for Go {
    fn into(self) -> TrafficLight {
        TrafficLight::Go(self)
    }
}
struct GetReady;
impl adar::prelude::StateTypes for GetReady {
    type States = TrafficLight;
    type Args = ();
    type Context = ();
}
impl Into<TrafficLight> for GetReady {
    fn into(self) -> TrafficLight {
        TrafficLight::GetReady(self)
    }
}
struct StopIfSafe;
impl adar::prelude::StateTypes for StopIfSafe {
    type States = TrafficLight;
    type Args = ();
    type Context = ();
}
impl Into<TrafficLight> for StopIfSafe {
    fn into(self) -> TrafficLight {
        TrafficLight::StopIfSafe(self)
    }
}
struct Stop;
impl adar::prelude::StateTypes for Stop {
    type States = TrafficLight;
    type Args = ();
    type Context = ();
}
impl Into<TrafficLight> for Stop {
    fn into(self) -> TrafficLight {
        TrafficLight::Stop(self)
    }
}
impl adar::prelude::StateTypes for TrafficLight {
    type States = Self;
    type Args = ();
    type Context = ();
}
impl adar::prelude::State for TrafficLight {
    fn on_enter(&mut self, args: Option<&mut Self::Args>, context: &mut Self::Context) {
        match self {
            Self::Go(s) => Go::on_enter(s, args, context),
            Self::GetReady(s) => GetReady::on_enter(s, args, context),
            Self::StopIfSafe(s) => StopIfSafe::on_enter(s, args, context),
            Self::Stop(s) => Stop::on_enter(s, args, context),
            _ => {}
        }
    }
    fn on_update(
        &mut self,
        args: Option<&mut Self::Args>,
        context: &mut Self::Context,
    ) -> Option<Self::States> {
        match self {
            Self::Go(s) => Go::on_update(s, args, context),
            Self::GetReady(s) => GetReady::on_update(s, args, context),
            Self::StopIfSafe(s) => StopIfSafe::on_update(s, args, context),
            Self::Stop(s) => Stop::on_update(s, args, context),
            _ => None,
        }
    }
    fn on_leave(&mut self, args: Option<&mut Self::Args>, context: &mut Self::Context) {
        match self {
            Self::Go(s) => Go::on_leave(s, args, context),
            Self::GetReady(s) => GetReady::on_leave(s, args, context),
            Self::StopIfSafe(s) => StopIfSafe::on_leave(s, args, context),
            Self::Stop(s) => Stop::on_leave(s, args, context),
            _ => {}
        }
    }
}
...
```

</details>

## Reflect Enum

Reflects information about the enum and its variants.

### Features

- Reflects the underlying type (see [ReflectEnum::Type](crate::enums::ReflectEnum::Type))
  - You may define your own repr (e.g. `#[repr(u8)]`)
- Reflects the name and value, or iterates over enum variants (see [ReflectEnum::variants](crate::enums::ReflectEnum::variants),[EnumVariant](crate::enums::EnumVariant))
- Number of variants (see [ReflectEnum::count](crate::enums::ReflectEnum::count))
- Name of the enum (see [ReflectEnum::name](crate::enums::ReflectEnum::name))

### Example

```rust
use adar::prelude::*;

#[ReflectEnum]
#[repr(u32)]
#[derive(Debug)]
enum MyEnum {
    Value1 = 33,
    Value2(i32),
    Value3 { a: String },
}

fn main() {
    println!("Variants count: {}", MyEnum::count());
    for variant in MyEnum::variants() {
        println!("{}, {:?}", variant.name, variant.value,);
    }
}
```

<details>
<summary>Click to see the output</summary>
<code><b>>> cargo run --example reflect_enum</b></code>

```ignore
Variants count: 3
Value1, Some(Value1)
Value2, None
Value3, None
```

</details>

<details>
<summary>Click to see the generated code</summary>
<code><b>>> cargo expand --example reflect_enum</b></code>

```rust,ignore
...
impl adar::prelude::ReflectEnum for MyEnum {
    type Type = u32;
    fn variants() -> &'static [adar::prelude::EnumVariant<MyEnum>] {
        const VARIANTS: &[adar::prelude::EnumVariant<MyEnum>] = &[
            EnumVariant::new("Value1", Some(MyEnum::Value1)),
            EnumVariant::new("Value2", None),
            EnumVariant::new("Value3", None),
        ];
        VARIANTS
    }
    fn count() -> usize {
        3usize
    }
    fn name(&self) -> &'static str {
        match self {
            Self::Value1 { .. } => "Value1",
            Self::Value2 { .. } => "Value2",
            Self::Value3 { .. } => "Value3",
        }
    }
}
...
```

</details>

## Enum Trait Deref

Enables you to access a trait through an enum whose named variants implement the same trait.

### Features

- Deref implementation (see [EnumTraitDeref](macros::EnumTraitDeref))
- DerefMut implementation (see [EnumTraitDerefMut](macros::EnumTraitDerefMut), also implements [EnumTraitDeref](macros::EnumTraitDeref) trait)

### Example

```rust
use adar::prelude::*;

trait MyTrait {
    fn my_func(&self);
}

#[EnumTraitDeref(MyTrait)]
enum MyEnum {
    A(A),
    B(B),
}

#[derive(Clone)]
struct A;

#[derive(Clone)]
struct B;

impl MyTrait for A {
    fn my_func(&self) {
        println!("Hello A");
    }
}

impl MyTrait for B {
    fn my_func(&self) {
        println!("Hello B");
    }
}

fn main() {
    for e in [MyEnum::A(A), MyEnum::B(B)] {
        e.my_func();
    }
}
```

<details>
<summary>Click to see the output</summary>
<code><b>>> cargo run --example enum_trait_deref</b></code>

```ignore
Hello A
Hello B
```

</details>
<details>
<summary>Click to see the generated code</summary>
<code><b>>> cargo expand --example enum_trait_deref</b></code>

```rust,ignore
...
impl ::core::ops::Deref for MyEnum {
    type Target = dyn MyTrait;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::A(v) => v as &Self::Target,
            Self::B(v) => v as &Self::Target,
        }
    }
}
...
```

</details>
