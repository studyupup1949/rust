use anti_gravity_qy::{Body, Simulation, get_resource_link};

fn main() {
    println!("--- Anti-gravity-qy Simulation ---");

    // 1. Initialize Simulation with theoretical anti-gravity field
    let mut sim = Simulation::new(1.5);
    println!("Initialized simulation with field strength: 1.5x");

    // 2. Add some test bodies
    let body1 = Body::new(100.0, 0.0, 0.0, 0.0);    // 100kg at sea level
    let body2 = Body::new(250.0, 10.0, 5.0, 50.0);  // 250kg at 50m altitude
    
    sim.add_body(body1);
    sim.add_body(body2);
    
    println!("Initial State:");
    for (i, b) in sim.bodies.iter().enumerate() {
        println!("  Body {}: {}kg at {}m altitude (PE: {} J)", i, b.mass, b.z, b.potential_energy());
    }

    // 3. Run simulation steps
    println!("\nRunning 10 simulation steps...");
    for _ in 0..10 {
        sim.simulate_step();
    }

    println!("Final State:");
    for (i, b) in sim.bodies.iter().enumerate() {
        println!("  Body {}: New altitude: {:.2}m", i, b.z);
    }

    // 4. Resource links
    println!("\nFor more specialized modules, see: {}", get_resource_link("/modules"));
    println!("Full documentation: https://antigravity.google/");
}