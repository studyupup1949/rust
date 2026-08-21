use actflow::{ChannelEvent, ChannelOptions, EngineBuilder, WorkflowModel};

fn main() {
    let engine = EngineBuilder::new().build().unwrap();

    engine.launch();

    let text = include_str!("./workflow.json");

    let workflow_model = WorkflowModel::from_json(text).unwrap();

    engine.deploy(&workflow_model).unwrap();

    let process = engine.build_process(&workflow_model.id).unwrap();
    let pid = process.id().to_string();

    ChannelEvent::channel(engine.channel(), ChannelOptions::with_pid(pid.to_owned())).on_complete(move |pid| {
        println!("Workflow completed, pid: {}", pid);
    });

    ChannelEvent::channel(engine.channel(), ChannelOptions::with_pid(pid.to_owned())).on_error(move |e| {
        println!("Workflow failed: {:?}", e);
    });

    engine.run_process(process.clone()).unwrap();

    loop {
        if process.is_complete() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    let outputs: serde_json::Value = process.get_outputs().into();
    println!("Outputs: {:#?}", outputs);
}
