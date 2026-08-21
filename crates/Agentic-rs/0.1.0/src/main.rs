use lib::{Agent, Cognitive, Message, Role};

pub mod lib;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Version 1.0");
    static SUMMARIZERDESCRIPTION: &str = r#"
    You are an Harward business review who has 30 years of experience edituaial rols.
    You ar excelent in english and business acdamic reseasrch. 
    Your role is make very brid short business case out of next texts:
    "#;
    static ARTICLE: &str = r#"
        The Biden administration announced a series of new financial sanctions Wednesday aimed at interrupting the fast-growing technological links between China and Russia that American officials believe are behind a broad effort to rebuild and modernize Russia’s military during its war with Ukraine.
    
        The actions were announced just as President Biden was leaving the country for a meeting in Italy of the Group of 7 industrialized economies, where a renewed effort to degrade the Russian economy will be at the top of his agenda.
        
        The effort has grown far more complicated in the past six or eight months after China, which previously had sat largely on the sidelines, has stepped up its shipments of microchips, optical systems for drones and components for advanced weaponry, U.S. officials said. But so far Beijing appears to have heeded Mr. Biden’s warning against shipping weapons to Russia, even as the United States and NATO continue to arm Ukraine.
        
        Announcing the new sanctions, Treasury Secretary Janet L. Yellen said in a statement that “Russia’s war economy is deeply isolated from the international financial system, leaving the Kremlin’s military desperate for access to the outside world.”
            "#;
    let small_llm = Cognitive::new("qwen2:72b".to_string());
    let profile = Message::new(SUMMARIZERDESCRIPTION.to_string(), Role::AGENT);
    let hamze = Agent::new("hamze".to_string(), profile, small_llm);
    let task = format!("Make summery of following text: {}", ARTICLE);

    println!("I am thinking... ");
    println!("Please wait. it could take while!");
    let final_reflection = hamze.execute(task).await;

    println!("summery: {:#?}", final_reflection);

    Ok(())
}
