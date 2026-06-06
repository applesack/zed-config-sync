use anyhow::Result;
use chrono::NaiveDateTime;

use crate::gist;

pub fn run() -> Result<()> {
    let client = gist::client_from_config()?;
    let history = client.get_history()?;

    let mut entries: Vec<(NaiveDateTime, String)> = history
        .into_iter()
        .filter_map(|(id, machine)| {
            let dt = NaiveDateTime::parse_from_str(&id, "cfg_%Y-%m-%d_%H-%M").ok()?;
            Some((dt, machine))
        })
        .collect();

    if entries.is_empty() {
        println!("No history records found.");
        return Ok(());
    }

    entries.sort_by_key(|(dt, _)| *dt);

    for (dt, machine) in &entries {
        let name = if machine.is_empty() {
            "unknown"
        } else {
            machine
        };
        println!("{}    {}", dt.format("%Y-%m-%d_%H:%M"), name);
    }

    Ok(())
}
