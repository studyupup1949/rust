use admin_config::*;

#[test]
fn test_app_config() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::load()?;
    dbg!(config);
    Ok(())
}
