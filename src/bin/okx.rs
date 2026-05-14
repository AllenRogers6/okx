use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{Shell, generate};
use okx::{db::Database, open_with_app, shell_init};

#[derive(Parser)]
#[command(name = "okx", about = "Track and manage frequently accessed files")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add {
        path: String,
    },

    Query {
        pattern: String,
        #[arg(short, long, default_value = "1")]
        top: usize,
    },

    Open {
        pattern: String,
    },

    Remove {
        path: String,
    },

    Clean,

    List,

    ShellInit {
        shell: String,
    },

    Completions {
        shell: Shell,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::ShellInit { shell } => {
            println!("{}", shell_init(shell));
            return Ok(());
        }
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            generate(*shell, &mut cmd, "okx", &mut std::io::stdout());
            return Ok(());
        }
        _ => {}
    }

    let db = Database::new()?;

    match cli.command {
        Commands::Add { path } => {
            let abs =
                std::fs::canonicalize(&path).unwrap_or_else(|_| std::path::PathBuf::from(&path));
            let abs_str = abs.to_string_lossy();
            db.add(&abs_str)?;
            println!("Added: {}", abs.display());
        }

        Commands::Query { pattern, top } => {
            let results = db.query_many(&pattern, top)?;
            if results.is_empty() {
                eprintln!("No match found.");
                std::process::exit(1);
            }
            for (path, score) in &results {
                if top > 1 {
                    println!("{:.2}\t{}", score, path);
                } else {
                    println!("{}", path);
                }
            }
        }

        Commands::Open { pattern } => match db.query(&pattern)? {
            Some((path, _)) => {
                db.add(&path)?;
                open_with_app(&path);
            }
            None => {
                eprintln!("No matching file for '{}'", pattern);
                std::process::exit(1);
            }
        },

        Commands::Remove { path } => {
            let abs =
                std::fs::canonicalize(&path).unwrap_or_else(|_| std::path::PathBuf::from(&path));
            let abs_str = abs.to_string_lossy();
            if db.remove(&abs_str)? {
                println!("Removed: {}", abs.display());
            } else {
                eprintln!("Not found in database: {}", abs.display());
                std::process::exit(1);
            }
        }

        Commands::Clean => {
            let n = db.clean()?;
            println!("Pruned {} stale entries.", n);
        }

        Commands::List => {
            for path in db.list_all()? {
                println!("{}", path);
            }
        }

        Commands::ShellInit { .. } | Commands::Completions { .. } => unreachable!(),
    }

    Ok(())
}
