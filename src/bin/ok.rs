use okx::{db::Database, interactive_pick, open_with_app};

fn main() -> anyhow::Result<()> {
    let arg = std::env::args().nth(1);
    let db = Database::new()?;

    match arg {
        None => {
            let all = db.list_all()?;
            if all.is_empty() {
                eprintln!("No files tracked yet. Run `okx add <file>` or `ok <file>` to start.");
                std::process::exit(1);
            }
            if let Some(path) = interactive_pick(&all) {
                db.add(&path)?;
                open_with_app(&path);
            }
        }

        Some(arg) => {
            if std::path::Path::new(&arg).exists() {
                let abs =
                    std::fs::canonicalize(&arg).unwrap_or_else(|_| std::path::PathBuf::from(&arg));
                let path = abs.to_string_lossy().to_string();
                db.add(&path)?;
                open_with_app(&path);
            } else {
                match db.query(&arg)? {
                    Some((path, _)) => {
                        db.add(&path)?;
                        open_with_app(&path);
                    }
                    None => {
                        eprintln!(
                            "No match for '{}'. Try `okx list` to see tracked files.",
                            arg
                        );
                        std::process::exit(1);
                    }
                }
            }
        }
    }

    Ok(())
}
