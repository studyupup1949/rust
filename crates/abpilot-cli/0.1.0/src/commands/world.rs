use abpilot_cc_sdk::{AbpilotClient, AuthMethod};
use crate::config::load_config;
use std::path::Path;
use tokio::fs;

pub async fn list() -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config()?;
    
    let mut client = AbpilotClient::new();
    client.mp_mut().set_auth(AuthMethod::api_key(config.apikey));
    
    let worlds = client.mp().list_worlds().await?;
    
    if worlds.is_empty() {
        println!("No worlds found");
    } else {
        println!("Worlds:");
        for world in worlds {
            println!("  - {} (ID: {})", world.name, world.world_id);
        }
    }
    
    Ok(())
}

pub async fn create(name: String) -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config()?;
    
    let mut client = AbpilotClient::new();
    client.mp_mut().set_auth(AuthMethod::api_key(config.apikey));
    
    let world = client.mp().create_world(&name).await?;
    
    println!("✓ World created:");
    println!("  Name: {}", world.name);
    println!("  World ID: {}", world.world_id);
    if let Some(secret) = world.secret {
        println!("  Secret: {}", secret);
    }
    
    Ok(())
}

pub async fn delete(world_id: String) -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config()?;
    
    let mut client = AbpilotClient::new();
    client.mp_mut().set_auth(AuthMethod::api_key(config.apikey));
    
    client.mp().delete_world(&world_id).await?;
    
    println!("✓ World deleted: {}", world_id);
    
    Ok(())
}

pub async fn get(world_id: String) -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config()?;
    
    let mut client = AbpilotClient::new();
    client.mp_mut().set_auth(AuthMethod::api_key(config.apikey));
    
    let world = client.mp().get_world(&world_id).await?;
    
    println!("World Details:");
    println!("  Name: {}", world.name);
    println!("  World ID: {}", world.world_id);
    
    Ok(())
}

pub async fn reset_secret(world_id: String) -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config()?;
    
    let mut client = AbpilotClient::new();
    client.mp_mut().set_auth(AuthMethod::api_key(config.apikey));
    
    let world = client.mp().reset_world_secret(&world_id).await?;
    
    println!("✓ World secret reset:");
    println!("  World ID: {}", world.world_id);
    if let Some(secret) = world.secret {
        println!("  New Secret: {}", secret);
    }
    
    Ok(())
}

pub async fn upload(world_id: String, files: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config()?;
    
    let mut client = AbpilotClient::new();
    client.mp_mut().set_auth(AuthMethod::api_key(config.apikey));
    
    println!("Getting upload URLs for {} files...", files.len());
    let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();
    let urls = client.mp().get_world_upload_urls(&world_id, &file_refs).await?;
    
    let http_client = reqwest::Client::new();
    
    for (file_path, upload_url) in files.iter().zip(urls.iter()) {
        let path = Path::new(file_path);
        if !path.exists() {
            eprintln!("✗ File not found: {}", file_path);
            continue;
        }
        
        println!("Uploading {}...", file_path);
        let content = fs::read(path).await?;
        
        let response = http_client
            .put(upload_url)
            .header("Content-Type", "application/octet-stream")
            .body(content)
            .send()
            .await?;
        
        if response.status().is_success() {
            println!("✓ Uploaded: {}", file_path);
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            eprintln!("✗ Failed to upload {}: {} - {}", file_path, status, body);
        }
    }
    
    Ok(())
}

pub async fn download(world_id: String, files: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config()?;
    
    let mut client = AbpilotClient::new();
    client.mp_mut().set_auth(AuthMethod::api_key(config.apikey));
    
    println!("Getting download URLs for {} files...", files.len());
    let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();
    let urls = client.mp().get_world_download_urls(&world_id, &file_refs).await?;
    
    let http_client = reqwest::Client::new();
    
    for (file_path, download_url) in files.iter().zip(urls.iter()) {
        println!("Downloading {}...", file_path);
        
        let response = http_client
            .get(download_url)
            .send()
            .await?;
        
        if response.status().is_success() {
            let content = response.bytes().await?;
            let path = Path::new(file_path);
            
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).await?;
            }
            
            fs::write(path, content).await?;
            println!("✓ Downloaded: {}", file_path);
        } else {
            eprintln!("✗ Failed to download {}: {}", file_path, response.status());
        }
    }
    
    Ok(())
}
