mod db;

use clap::{Parser, Subcommand};
use db::Database;
use mime_guess::mime;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add { path: String },
    Query { pattern: String },
    Open { pattern: String },
    List,
}

fn is_text_file(path: &str) -> bool {
    let guess = mime_guess::from_path(path);
    if let Some(mime) = guess.first() {
        let type_ = mime.type_();
        let subtype = mime.subtype().as_str();
        if type_ == mime::TEXT {
            return true;
        }
        if type_ == mime::APPLICATION {
            return matches!(
                subtype,
                "json"
                    | "xml"
                    | "javascript"
                    | "x-javascript"
                    | "ecmascript"
                    | "csv"
                    | "toml"
                    | "yaml"
                    | "yml"
                    | "graphql"
                    | "sql"
                    | "x-sh"
                    | "x-httpd-php"
            );
        }
        return matches!(
            subtype,
            "html" | "css" | "markdown" | "x-markdown" | "plain"
        );
    }
    match std::path::Path::new(path)
        .extension()
        .and_then(|s| s.to_str())
    {
        Some(ext) => matches!(
            ext.to_ascii_lowercase().as_str(),
            "txt"
                | "md"
                | "rs"
                | "c"
                | "cpp"
                | "h"
                | "py"
                | "js"
                | "ts"
                | "java"
                | "go"
                | "html"
                | "css"
                | "json"
                | "xml"
                | "yaml"
                | "yml"
                | "toml"
                | "csv"
                | "ini"
                | "sh"
        ),
        None => false,
    }
}

fn open_with_app(path: &str) {
    if is_text_file(path) {
        let default_editor = if cfg!(target_os = "windows") {
            "notepad"
        } else {
            "vim"
        };

        let editor = std::env::var("FILERUN_EDITOR")
            .or_else(|_| std::env::var("EDITOR"))
            .or_else(|_| std::env::var("VISUAL"))
            .unwrap_or_else(|_| default_editor.into());

        std::process::Command::new(&editor)
            .arg(path)
            .status()
            .unwrap_or_else(|_| panic!("failed to launch editor: {}", editor));
    } else {
        open::that(path).unwrap_or_else(|e| eprintln!("failed to open file: {e}"));
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let db = Database::new()?;

    match cli.command {
        Commands::Add { path } => {
            let abs =
                std::fs::canonicalize(&path).unwrap_or_else(|_| std::path::PathBuf::from(&path));
            db.add(abs.to_str().unwrap())?;
            println!("Added: {}", abs.display());
        }
        Commands::Query { pattern } => {
            if let Some((path, _)) = db.query(&pattern)? {
                println!("{}", path);
            } else {
                eprintln!("No match found.");
                std::process::exit(1);
            }
        }
        Commands::Open { pattern } => match db.query(&pattern)? {
            Some((path, _score)) => {
                db.add(&path)?;
                open_with_app(&path);
            }
            None => {
                eprintln!("No matching file for '{}'", pattern);
                std::process::exit(1);
            }
        },
        Commands::List => {
            for path in db.list_all()? {
                println!("{}", path);
            }
        }
    }
    Ok(())
}
