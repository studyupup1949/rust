use abpilot_cc_sdk::{AbpilotClient, AuthMethod};
use crate::config::load_config;
use std::path::Path;
use tokio::fs;

pub async fn list() -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config()?;
    
    let mut client = AbpilotClient::new();
    client.mp_mut().set_auth(AuthMethod::api_key(config.apikey));
    
    let apps = client.mp().list_apps().await?;
    
    if apps.is_empty() {
        println!("No apps found");
    } else {
        println!("Apps:");
        for app in apps {
            println!("  - {} (ID: {})", app.name, app.app_id);
        }
    }
    
    Ok(())
}

pub async fn create(name: String) -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config()?;
    
    let mut client = AbpilotClient::new();
    client.mp_mut().set_auth(AuthMethod::api_key(config.apikey));
    
    let app = client.mp().create_app(&name).await?;
    
    println!("✓ App created:");
    println!("  Name: {}", app.name);
    println!("  App ID: {}", app.app_id);
    if let Some(secret) = app.secret {
        println!("  Secret: {}", secret);
    }
    
    Ok(())
}

pub async fn delete(app_id: String) -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config()?;
    
    let mut client = AbpilotClient::new();
    client.mp_mut().set_auth(AuthMethod::api_key(config.apikey));
    
    client.mp().delete_app(&app_id).await?;
    
    println!("✓ App deleted: {}", app_id);
    
    Ok(())
}

pub async fn reset_secret(app_id: String) -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config()?;
    
    let mut client = AbpilotClient::new();
    client.mp_mut().set_auth(AuthMethod::api_key(config.apikey));
    
    let app = client.mp().reset_app_secret(&app_id).await?;
    
    println!("✓ App secret reset:");
    println!("  App ID: {}", app.app_id);
    if let Some(secret) = app.secret {
        println!("  New Secret: {}", secret);
    }
    
    Ok(())
}

pub async fn upload(app_id: String, files: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config()?;
    
    let mut client = AbpilotClient::new();
    client.mp_mut().set_auth(AuthMethod::api_key(config.apikey));
    
    println!("Getting upload URLs for {} files...", files.len());
    let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();
    let urls = client.mp().get_app_upload_urls(&app_id, &file_refs).await?;
    
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

pub async fn download(app_id: String, files: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config()?;
    
    let mut client = AbpilotClient::new();
    client.mp_mut().set_auth(AuthMethod::api_key(config.apikey));
    
    println!("Getting download URLs for {} files...", files.len());
    let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();
    let urls = client.mp().get_app_download_urls(&app_id, &file_refs).await?;
    
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
