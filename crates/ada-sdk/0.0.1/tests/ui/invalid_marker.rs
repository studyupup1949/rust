use ada_sdk::PrincipalEvents;

fn invalid(events: PrincipalEvents) {
    events.on_memory_deleted(|_| {});
}

fn main() {}
