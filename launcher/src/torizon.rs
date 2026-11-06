//! Portable entry point for the Torizon fork. Upstream crate and save identifiers stay stable.
use std::fs;
use std::path::Path;

pub const VERSION: &str = "torizon-0.1.0";

fn copy_tree(source: &Path, destination: &Path) -> std::io::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let target = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

/// Configure a separate portable home before the launcher or its CLI starts any threads.
pub fn initialize() -> Result<(), String> {
    let executable = std::env::current_exe().map_err(|e| e.to_string())?;
    let root = executable.parent().ok_or("No executable folder")?;
    for (key, value) in [
        ("HORIZON_LAUNCHER_HOME", root.join("data/launcher")),
        ("HORIZON_HOME", root.join("game")),
    ] {
        if std::env::var_os(key).is_none_or(|value| value.is_empty()) {
            std::env::set_var(key, value);
        }
    }
    std::env::set_var("HORIZON_LAUNCHER_NO_SELF_UPDATE", "1");
    std::env::set_var("HORIZON_TELEMETRY_URL", "off");
    let data = crate::backend::data_dir();
    fs::create_dir_all(&data).map_err(|e| e.to_string())?;
    let bundled = root.join("runtime");
    let installed = data.join("versions").join(VERSION);
    let marker = installed.join(".torizon-bundled");
    if bundled.join("research-lock.json").is_file() && !marker.is_file() {
        copy_tree(&bundled, &installed).map_err(|e| e.to_string())?;
        fs::write(installed.join(".installed"), b"Torizon 0.1.0\n").map_err(|e| e.to_string())?;
        fs::write(&marker, b"Torizon 0.1.0\n").map_err(|e| e.to_string())?;
    }
    if installed.join("horizon-client.exe").is_file() {
        let mut settings = crate::backend::Settings::load();
        settings.selected = Some(VERSION.into());
        settings.save()?;
    }
    if !data.join("accounts.json").is_file() {
        let mut accounts = crate::account::Accounts::default();
        accounts.play_as_with(
            &crate::account::MemoryStore::default(),
            crate::account::Player::Offline("Player".into()),
            true,
        )?;
        accounts.save()?;
    }
    Ok(())
}
