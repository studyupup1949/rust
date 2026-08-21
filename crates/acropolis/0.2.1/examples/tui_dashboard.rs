use acropolis::{Node, NodeConfig};

fn main() -> std::io::Result<()> {
    let node = Node::new(NodeConfig::default()).expect("default config is valid");
    acropolis::dashboard::render_startup_dashboard(&node.startup_plan())
}
