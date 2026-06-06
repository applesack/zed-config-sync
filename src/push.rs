use std::collections::HashMap;
use std::fs;

use anyhow::Result;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;
use chrono::Local;

use crate::gist;
use crate::util::{Workspace, copy_dir_all, create_zip, machine_name, zed_dir};

const MAX_CONFIGS: usize = 5;

pub fn run() -> Result<()> {
    let client = gist::client_from_config()?;
    let ws = Workspace::prepare()?;

    let cfg_files = client.list_cfg_files()?;

    if let Some(latest) = cfg_files.last() {
        println!("Downloading latest config: {}", latest);
        ws.download_and_extract(&client, latest)?;
    }

    let zed = zed_dir()?;
    if zed.exists() {
        copy_dir_all(&zed, &ws.temp_dir)?;
    } else {
        println!("Warning: Zed config directory not found: {}", zed.display());
    }

    let filename = Local::now().format("cfg_%Y-%m-%d_%H-%M").to_string();
    let zip_path = ws.zip_dir.join(format!("{}.zip", &filename));
    println!("Packaging as {}...", filename);
    create_zip(&ws.temp_dir, &zip_path)?;

    let zip_b64 = B64.encode(fs::read(&zip_path)?);
    let mut history = client.get_history()?;

    if cfg_files.len() >= MAX_CONFIGS {
        let oldest = cfg_files[0].clone();
        println!("Config limit reached, removing oldest: {}", oldest);
        history.remove(&oldest);
        let mut ops: HashMap<String, Option<String>> = HashMap::new();
        ops.insert(oldest, None);
        ops.insert(
            "history.json".to_string(),
            Some(serde_json::to_string_pretty(&history)?),
        );
        client.patch_files(&ops)?;
    }

    history.insert(filename.clone(), machine_name());
    println!("Uploading {}...", filename);
    let mut ops: HashMap<String, Option<String>> = HashMap::new();
    ops.insert(filename, Some(zip_b64));
    ops.insert(
        "history.json".to_string(),
        Some(serde_json::to_string_pretty(&history)?),
    );
    client.patch_files(&ops)?;

    ws.cleanup()?;
    println!("Done! Config pushed successfully.");
    Ok(())
}
