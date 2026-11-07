//! Portable entry point for the Torizon fork. Upstream crate and save identifiers stay stable.
use std::fs;
use std::path::Path;

pub const VERSION: &str = "torizon-0.1.1";

/// Release EXEs carry a ZIP overlay containing only our runtime. ZIP offsets are
/// relative to the appended archive, so ZipArchive also accepts the PE prefix.
fn unpack_runtime(executable: &Path, destination: &Path) -> Result<bool, String> {
    let file = fs::File::open(executable).map_err(|e| e.to_string())?;
    let mut archive = match zip::ZipArchive::new(file) {
        Ok(archive) => archive,
        // Development binaries and the executable inside the portable ZIP have
        // no overlay; their runtime is supplied as a sibling directory.
        Err(zip::result::ZipError::InvalidArchive(_)) => return Ok(false),
        Err(error) => return Err(error.to_string()),
    };
    for name in [
        "research-lock.json",
        "horizon-client.exe",
        "horizon-prepare.exe",
    ] {
        archive.by_name(name).map_err(|e| e.to_string())?;
    }
    archive.extract(destination).map_err(|e| e.to_string())?;
    Ok(true)
}

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
    if !marker.is_file() || !installed.join("horizon-client.exe").is_file() {
        let ready = if bundled.join("research-lock.json").is_file() {
            copy_tree(&bundled, &installed).map_err(|e| e.to_string())?;
            true
        } else {
            unpack_runtime(&executable, &installed)?
        };
        if ready {
            fs::write(installed.join(".installed"), b"Torizon 0.1.1\n")
                .map_err(|e| e.to_string())?;
            fs::write(&marker, b"Torizon 0.1.1\n").map_err(|e| e.to_string())?;
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn pe_overlay_installs_runtime_and_plain_exe_does_not() {
        let root = std::env::temp_dir().join(format!("torizon-overlay-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let exe = root.join("launcher.exe");
        let destination = root.join("installed");
        let mut file = fs::File::create(&exe).unwrap();
        file.write_all(b"MZ\0\0test PE prefix").unwrap();
        assert!(!unpack_runtime(&exe, &destination).unwrap());
        let mut archive = zip::ZipWriter::new(file);
        for (name, contents) in [
            ("research-lock.json", "{\"version\":\"26.3\"}"),
            ("horizon-client.exe", "native client"),
            ("horizon-prepare.exe", "native preparer"),
            ("tools/nested/exporter.txt", "original tool"),
        ] {
            archive
                .start_file(name, zip::write::SimpleFileOptions::default())
                .unwrap();
            archive.write_all(contents.as_bytes()).unwrap();
        }
        drop(archive.finish().unwrap());
        assert!(unpack_runtime(&exe, &destination).unwrap());
        assert_eq!(
            fs::read(destination.join("horizon-client.exe")).unwrap(),
            b"native client"
        );
        assert_eq!(
            fs::read(destination.join("tools/nested/exporter.txt")).unwrap(),
            b"original tool"
        );
        fs::remove_dir_all(root).unwrap();
    }
}
