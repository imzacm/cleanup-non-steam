use anyhow::Context;

fn main() -> anyhow::Result<()> {
    let steam_dir = steamlocate::SteamDir::locate().context("Locate steam installation")?;
    println!("Steam installation - {}", steam_dir.path().display());

    let userdata_path = steam_dir.path().join("userdata");
    let shortcut_file_iter = std::fs::read_dir(&userdata_path)
        .context("Read userdata directory")?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let mut path = entry.path();
            path.push("config");
            path.push("shortcuts.vdf");
            if path.is_file() {
                path.shrink_to_fit();
                Some(path)
            } else {
                None
            }
        });

    for shortcut_path in shortcut_file_iter {
        println!("Shortcuts file: {}", shortcut_path.display());
        let content = std::fs::read(&shortcut_path)
            .with_context(|| format!("Read shortcuts file - {}", shortcut_path.display()))?;

        let mut shortcuts = steam_shortcuts_util::parse_shortcuts(&content)
            .map_err(|e| anyhow::anyhow!("Parse shortcuts - {} : {e}", shortcut_path.display()))?;
        let init_len = shortcuts.len();

        // let shortcut_bytes = steam_shortcuts_util::shortcuts_to_bytes(&shortcuts);
        // assert_eq!(content.len(), shortcut_bytes.len());
        // assert_eq!(shortcut_bytes, content);

        shortcuts.retain(|shortcut| {
            let exe_path = unquote_str(shortcut.exe);
            println!(
                "Shortcut: {} - {exe_path} - {}",
                shortcut.app_name, shortcut.shortcut_path
            );

            if shortcut.is_hidden {
                return true;
            }

            if !std::fs::exists(exe_path).unwrap_or(false) {
                println!(
                    "Deleting shortcut to non-existent path - {}: {exe_path}",
                    shortcut.app_name
                );
                return false;
            }

            true
        });

        if shortcuts.len() != init_len {
            let backup_file = shortcut_path.with_extension(".vdf.bak");
            std::fs::copy(&shortcut_path, &backup_file).with_context(|| {
                format!(
                    "Backup {} to {}",
                    shortcut_path.display(),
                    backup_file.display()
                )
            })?;

            let shortcut_bytes = steam_shortcuts_util::shortcuts_to_bytes(&shortcuts);
            std::fs::write(&shortcut_path, shortcut_bytes)
                .with_context(|| format!("Write shortcuts file - {}", shortcut_path.display()))?;
        }
    }

    Ok(())
}

fn unquote_str(s: &str) -> &str {
    s.trim_start_matches('"').trim_end_matches('"')
}
