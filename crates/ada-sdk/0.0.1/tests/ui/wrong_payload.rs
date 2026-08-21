use ada_sdk::PrincipalEvents;
use ada_sdk::proto::MemoryIngestStarted;

fn invalid(events: PrincipalEvents) {
    events.on_memory_ingest_finished(
        |_: MemoryIngestStarted| {},
    );
}

fn main() {}
