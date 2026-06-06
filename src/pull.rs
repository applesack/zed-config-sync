use anyhow::Result;

use crate::gist;
use crate::util::{Workspace, copy_dir_all, zed_dir};

pub fn run() -> Result<()> {
    let client = gist::client_from_config()?;
    let ws = Workspace::prepare()?;

    let cfg_files = client.list_cfg_files()?;
    let Some(latest) = cfg_files.last() else {
        println!("No cloud config available. Nothing to pull.");
        ws.cleanup()?;
        return Ok(());
    };

    println!("Downloading latest config: {}", latest);
    ws.download_and_extract(&client, latest)?;

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
    println!("Done! Config pulled from: {}", latest);
    Ok(())
}
