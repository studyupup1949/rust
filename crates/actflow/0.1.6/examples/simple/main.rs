use actflow::{ChannelEvent, ChannelOptions, EngineBuilder, WorkflowModel};

fn main() {
    let engine = EngineBuilder::new().build().unwrap();

    // Launch the engine
    engine.launch();

    let text = include_str!("./workflow.json");

    let workflow_model = WorkflowModel::from_json(text).unwrap();

    let process = engine.build_workflow_process(&workflow_model).unwrap();
    let pid = process.id();

    ChannelEvent::channel(engine.channel(), ChannelOptions::with_pid(pid.to_owned())).on_complete(move |pid| {
        println!("Workflow completed, pid: {}", pid);
    });

    ChannelEvent::channel(engine.channel(), ChannelOptions::with_pid(pid.to_owned())).on_error(move |e| {
        println!("Workflow failed: {:?}", e);
    });

    // Start the workflow process
    process.start();

    loop {
        if process.is_complete() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    let outputs: serde_json::Value = process.get_outputs().into();
    println!("Outputs: {:#?}", outputs);
}
