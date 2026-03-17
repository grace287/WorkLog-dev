mod cli;
mod connectors;
mod core;

use anyhow::Result;
use clap::{Parser, Subcommand};

use cli::{auth, portfolio, sync, task};
use core::output::print_error;

// ── Top-level CLI ─────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name = "worklog",
    version = "0.1.0",
    about = "Turn Git commits into a verified portfolio",
    long_about = None,
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Authenticate with GitHub PAT
    Init,

    /// Show authenticated GitHub login
    Whoami,

    /// Remove stored credentials
    Logout,

    /// Manage tasks
    Task {
        #[command(subcommand)]
        action: TaskCommands,
    },

    /// Scan git history and link commits to tasks
    Sync {
        /// Parse commits since this date (e.g. 2024-01-01 or 30d)
        #[arg(long)]
        since: Option<String>,

        /// Preview without writing changes
        #[arg(long)]
        dry_run: bool,

        /// Skip GitHub verification
        #[arg(long)]
        no_verify: bool,
    },

    /// Sync HEAD commit immediately
    Push,

    /// Show task-commit timeline
    Log,

    /// Publish portfolio to worklog.dev
    Publish {
        /// Visibility: public | unlisted | private
        #[arg(long, default_value = "public")]
        visibility: String,
    },

    /// Show portfolio status summary
    Status,

    /// Export portfolio locally
    Export {
        /// Format: md | json
        #[arg(long, short, default_value = "md")]
        format: String,
    },
}

#[derive(Subcommand)]
enum TaskCommands {
    /// Add a new task
    Add {
        /// Task title
        title: String,

        /// Project ID (overrides default_project from config)
        #[arg(long, short)]
        project: Option<String>,
    },

    /// List tasks
    Ls {
        /// Filter by project ID
        #[arg(long, short)]
        project: Option<String>,
    },

    /// Mark a task as done
    Done {
        /// Task key (e.g. PROJ-1)
        task_key: String,
    },

    /// Move a task to a different status
    Move {
        /// Task key (e.g. PROJ-1)
        task_key: String,
        /// New status: todo | doing | done
        status: String,
    },

    /// Manually link a task to a commit SHA
    Link {
        /// Task key (e.g. PROJ-1)
        task_key: String,
        /// Commit SHA
        sha: String,
    },
}

// ── Entry point ───────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        print_error(&format!("{:#}", e));
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => auth::run_init().await?,

        Commands::Whoami => auth::run_whoami().await?,

        Commands::Logout => auth::run_logout()?,

        Commands::Task { action } => match action {
            TaskCommands::Add { title, project } => {
                task::run_add(&title, project.as_deref())?
            }
            TaskCommands::Ls { project } => task::run_ls(project.as_deref())?,
            TaskCommands::Done { task_key } => task::run_done(&task_key)?,
            TaskCommands::Move { task_key, status } => {
                task::run_move(&task_key, &status)?
            }
            TaskCommands::Link { task_key, sha } => {
                task::run_link(&task_key, &sha)?
            }
        },

        Commands::Sync {
            since,
            dry_run,
            no_verify,
        } => sync::run_sync(since.as_deref(), dry_run, no_verify).await?,

        Commands::Push => sync::run_push().await?,

        Commands::Log => sync::run_log()?,

        Commands::Publish { visibility } => {
            portfolio::run_publish(Some(&visibility)).await?
        }

        Commands::Status => portfolio::run_status()?,

        Commands::Export { format } => portfolio::run_export(Some(&format))?,
    }

    Ok(())
}
