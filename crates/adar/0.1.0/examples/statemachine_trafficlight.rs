/// Mermaid diagram: https://mermaid.live/edit#pako:eNp1kVFPgzAUhf8Kub4ZtkiBMfpgYpxZlixithmjYpYKF0ZW6FKKOpf9d8sYTjHep3tvv3NOm-4gEjEChVIxhaOMpZLlvTcSFoauiLOyHGFiSIyNJOOcnjFCzEhwIen7KlPY4VKJWBxJwlrylbNo3SG3yLl4b027aAM_n78Yvd6lMVdi02zqjlJaX6c-GKOaIYu3BjXGwXK-CO6Wo_vZ1WIS3DZ8C2jNMfAgE1rweDOdBg9dXmiyeUQbPEnmLMF_E07I74x6_zcFTEhlFgNVskITcpQ5q0fY1W4hqBXmGALVbczkOoSw2GvNhhVPQuStTIoqXQFNGC_1VG3i09d9byUWMcprURUKqDvwDyZAd_AB1PbtvmUPLddziUM81zJhqyGr79ue49nEcayB5wzdvQmfh9iL_tBz_J-1_wJ7ja8f
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
