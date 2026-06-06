use anyhow::Result;
use chrono::NaiveDateTime;

use crate::gist;
use crate::util::{Workspace, copy_dir_all, zed_dir};

pub fn run(config_id: &str) -> Result<()> {
    let dt = NaiveDateTime::parse_from_str(config_id, "%Y-%m-%d_%H:%M").map_err(|_| {
        anyhow::anyhow!(
            "Invalid config ID: '{}'\nExpected format: YYYY-MM-DD_HH:mm (e.g. 2025-06-06_14:30)",
            config_id
        )
    })?;
    let gist_filename = dt.format("cfg_%Y-%m-%d_%H-%M").to_string();

    let client = gist::client_from_config()?;
    let ws = Workspace::prepare()?;

    let cfg_files = client.list_cfg_files()?;
    if !cfg_files.contains(&gist_filename) {
        ws.cleanup()?;
        anyhow::bail!("Config '{}' not found in gist.", config_id);
    }

    println!("Downloading config: {}", gist_filename);
    ws.download_and_extract(&client, &gist_filename)?;

    let zed = zed_dir()?;
    if !zed.exists() {
        ws.cleanup()?;
        anyhow::bail!(
            "Zed config directory not found: {}\n\
             Please make sure Zed is installed and has been launched at least once.",
            zed.display()
        );
    }

    println!("Applying config...");
    copy_dir_all(&ws.temp_dir, &zed)?;

    ws.cleanup()?;
    println!("Done! Restored config: {}", config_id);
    Ok(())
}
