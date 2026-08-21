# Agents Package

The `agents` package provides a set of tools for creating and managing agents capable of processing text and performing
specified tasks based on a given role. It leverages asynchronous Rust programming with Tokio for efficient task
execution.

## Usage

This package allows you to create instances of `Agent` with specific roles and execute tasks using natural language
understanding. Below is a quick start guide on using the package:

### Example: Summarizing Text

The following example demonstrates how to set up and use an `Agent` to summarize a lengthy article. The text used in
this example is arbitrarily chosen and copied from the web purely for demonstration purposes.

**main.rs**

```rust
extern crate agents;

use agents::{Agent, Cognitive, Message, Role};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Version 1.0");

    // Description for the agent's persona
    static SUMMARIZERDESCRIPTION: &str = r#"
    You are an Harvard Business Review editor with 30 years of experience in editorial roles.
    You are excellent in English and business academic research.
    Your role is to make a very brief short business case out of the next texts:
    "#;

    // Sample article text
    static ARTICLE: &str = r#"
    The Biden administration announced a series of new financial sanctions Wednesday aimed at interrupting the fast-growing technological links between China and Russia that American officials believe are behind a broad effort to rebuild and modernize Russia’s military during its war with Ukraine.

    The actions were announced just as President Biden was leaving the country for a meeting in Italy of the Group of 7 industrialized economies, where a renewed effort to degrade the Russian economy will be at the top of his agenda.

    The effort has grown far more complicated in the past six or eight months after China, which previously had sat largely on the sidelines, has stepped up its shipments of microchips, optical systems for drones and components for advanced weaponry, U.S. officials said. But so far Beijing appears to have heeded Mr. Biden’s warning against shipping weapons to Russia, even as the United States and NATO continue to arm Ukraine.

    Announcing the new sanctions, Treasury Secretary Janet L. Yellen said in a statement that “Russia’s war economy is deeply isolated from the international financial system, leaving the Kremlin’s military desperate for access to the outside world.”
    "#;

    // Initialize cognitive resources
    let small_llm = Cognitive::new("qwen2:72b".to_string());

    // Create agent profile
    let profile = Message::new(SUMMARIZERDESCRIPTION.to_string(), Role::AGENT);

    // Create agent
    let hamze = Agent::new("Hamze".to_string(), profile, small_llm);

    // Task to summarize
    let task = format!("Make summary of the following text: {}", ARTICLE);

    println!("I am thinking... ");
    println!("Please wait, it could take a while!");

    // Execute task
    let final_reflection = hamze.execute(task).await;

    println!("Summary: {:#?}", final_reflection);

    Ok(())
}
```

## Features

- **Cognitive Modeling**: Initialize agents with cognitive abilities to understand and generate natural language.
- **Asynchronous Execution**: Utilize async/await for non-blocking task execution.
- **Customizable Roles**: Define roles such as `AGENT`, `USER`, and `SYSTEM` for context-based behavior.

