// berri-recall - remembers your terminal commands so you don't have to
//
// This is the main entry point. Parses CLI args and dispatches to handlers.

use berri_recall_lib::{
    core::{ProjectDetector, Recorder},
    intelligence::Analyzer,
    shell::{HookInstaller, ShellDetector},
    Database, Result,
};
use std::env;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Grab whatever the user typed
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    let command = &args[1];

    match command.as_str() {
        "record" => handle_record(&args[2..]).await,
        "recent" => handle_recent(&args[2..]).await,
        "search" => handle_search(&args[2..]).await,
        "setup" => handle_setup(&args[2..]).await,
        "uninstall" => handle_uninstall(&args[2..]).await,
        "status" => handle_status().await,
        "analyze" => handle_analyze(&args[2..]).await,
        "suggest" => handle_suggest().await,
        "version" | "-v" | "--version" => {
            println!("berri-recall v{}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        "help" | "-h" | "--help" => {
            print_usage();
            Ok(())
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage();
            Ok(())
        }
    }
}

async fn handle_record(args: &[String]) -> Result<()> {
    // Parse flags and extract the actual command
    let mut command_parts = Vec::new();
    let mut exit_code: Option<i32> = None;
    let mut cwd_override: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--command" => {
                i += 1;
                if i < args.len() {
                    command_parts.push(args[i].clone());
                }
            }
            "--exit-code" => {
                i += 1;
                if i < args.len() {
                    exit_code = args[i].parse().ok();
                }
            }
            "--cwd" => {
                i += 1;
                if i < args.len() {
                    cwd_override = Some(args[i].clone());
                }
            }
            arg => command_parts.push(arg.to_string()),
        }
        i += 1;
    }

    if command_parts.is_empty() {
        // Sometimes shell hooks call us with nothing. Just ignore it.
        return Ok(());
    }

    let command_to_record = command_parts.join(" ");

    // Figure out where the user ran this from
    let cwd = if let Some(cwd_path) = cwd_override {
        std::path::PathBuf::from(cwd_path)
    } else {
        env::current_dir()?
    };

    let project_root = ProjectDetector::detect(&cwd)?;

    let db = get_database().await?;
    let recorder = Recorder::new(Arc::new(db));

    // Skip stuff we don't care about (passwords, env vars, etc)
    if recorder.should_ignore(&command_to_record) {
        return Ok(());
    }

    match recorder
        .record(
            &command_to_record,
            project_root.to_str().unwrap(),
            None,
            exit_code,
            None,
        )
        .await
    {
        Ok(_) => {} // worked fine, don't say anything
        Err(_) => {
            // failed but don't spam the terminal. nobody likes that.
        }
    }

    Ok(())
}

async fn handle_recent(args: &[String]) -> Result<()> {
    let limit = args
        .get(0)
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(10);

    let db = get_database().await?;
    let cwd = env::current_dir()?;
    let project_root = ProjectDetector::detect(&cwd).ok();

    let commands = db
        .get_recent_commands(project_root.as_ref().and_then(|p| p.to_str()), limit)
        .await?;

    if commands.is_empty() {
        println!("No commands found.");
    } else {
        println!("\nRecent commands:");
        println!("{}", "=".repeat(60));
        for (i, cmd) in commands.iter().enumerate() {
            let status = if let Some(code) = cmd.exit_code {
                if code == 0 {
                    "✓"
                } else {
                    "✗"
                }
            } else {
                " "
            };
            println!(
                "{:3}. {} {} (used {} times)",
                i + 1,
                status,
                cmd.command,
                cmd.usage_count
            );
        }
        println!("{}", "=".repeat(60));
    }

    Ok(())
}

async fn handle_search(args: &[String]) -> Result<()> {
    if args.is_empty() {
        eprintln!("Error: No search query provided");
        return Ok(());
    }

    let query = args.join(" ");
    let db = get_database().await?;
    let cwd = env::current_dir()?;
    let project_root = ProjectDetector::detect(&cwd).ok();

    let results = db
        .search_commands(&query, project_root.as_ref().and_then(|p| p.to_str()), 20)
        .await?;

    if results.is_empty() {
        println!("No commands found matching '{}'", query);
    } else {
        println!("\nFound {} command(s) matching '{}':", results.len(), query);
        println!("{}", "=".repeat(60));
        for (i, cmd) in results.iter().enumerate() {
            println!(
                "{:3}. {} (used {} times)",
                i + 1,
                cmd.command,
                cmd.usage_count
            );
        }
        println!("{}", "=".repeat(60));
    }

    Ok(())
}

async fn handle_setup(args: &[String]) -> Result<()> {
    let installer = HookInstaller::new()?;

    // Check for --all flag
    let install_all = args.iter().any(|arg| arg == "--all");

    if install_all {
        println!("Installing hooks for all detected shells...\n");
        match installer.install_all() {
            Ok(shells) => {
                println!("✓ Successfully installed hooks for:");
                for shell in shells {
                    println!("  - {}", shell);
                }
                println!("\n🎉 Setup complete! Restart your shell or run:");
                println!("   source ~/.bashrc   (for bash)");
                println!("   source ~/.zshrc    (for zsh)");
            }
            Err(e) => {
                eprintln!("✗ Setup failed: {}", e);
                return Err(e);
            }
        }
    } else {
        // Auto-detect and install for current shell
        println!("Detecting your shell...\n");
        match installer.install_auto() {
            Ok(shell) => {
                println!("✓ Detected shell: {}", shell);
                println!("✓ Hook installed successfully!\n");
                println!("🎉 Setup complete! Restart your shell or run:");
                use berri_recall_lib::shell::Shell;
                match shell {
                    Shell::Bash => println!("   source ~/.bashrc"),
                    Shell::Zsh => println!("   source ~/.zshrc"),
                    Shell::Fish => {
                        println!("   source ~/.config/fish/config.fish")
                    }
                    Shell::PowerShell => {
                        println!("   . $PROFILE")
                    }
                }
            }
            Err(e) => {
                eprintln!("✗ Setup failed: {}", e);
                eprintln!("\nTry running with --all flag to install for all shells:");
                eprintln!("   berri-recall setup --all");
                return Err(e);
            }
        }
    }

    Ok(())
}

async fn handle_uninstall(_args: &[String]) -> Result<()> {
    let installer = HookInstaller::new()?;

    println!("Uninstalling berri-recall hooks...\n");

    use berri_recall_lib::shell::Shell;

    let shells = vec![
        Shell::Bash,
        Shell::Zsh,
        Shell::Fish,
        Shell::PowerShell,
    ];

    for shell in shells {
        match installer.uninstall(shell) {
            Ok(()) => println!("✓ Uninstalled {} hook", shell),
            Err(e) => eprintln!("  (skipped {}: {})", shell, e),
        }
    }

    println!("\n✓ Uninstall complete!");
    println!("Note: Database (~/.berri-recall/) was not removed.");
    println!("To remove all data: rm -rf ~/.berri-recall");

    Ok(())
}

async fn handle_status() -> Result<()> {
    let installer = HookInstaller::new()?;
    let db = get_database().await?;
    let stats = db.stats().await?;

    println!("\nberri-recall Status");
    println!("{}", "=".repeat(60));

    // Shell hooks status
    println!("\nShell Hooks:");
    use berri_recall_lib::shell::Shell;
    for shell in &[
        Shell::Bash,
        Shell::Zsh,
        Shell::Fish,
        Shell::PowerShell,
    ] {
        let status = if installer.is_installed(*shell) {
            "✓ Installed"
        } else {
            "✗ Not installed"
        };
        println!("  {:<12} {}", format!("{}:", shell), status);
    }

    // Database stats
    println!("\nDatabase Statistics:");
    println!("  Commands:    {}", stats.total_commands);
    println!("  Patterns:    {}", stats.total_patterns);
    println!("  Suggestions: {}", stats.total_suggestions);

    // Current shell
    println!("\nCurrent Shell:");
    match ShellDetector::detect() {
        Ok(shell) => println!("  {}", shell),
        Err(_) => println!("  Unknown"),
    }

    println!("{}", "=".repeat(60));

    Ok(())
}

async fn handle_analyze(_args: &[String]) -> Result<()> {
    let db = Arc::new(get_database().await?);
    let analyzer = Analyzer::new(db);

    let cwd = env::current_dir()?;
    let project_root = ProjectDetector::detect(&cwd).ok();

    println!("\n🔍 Analyzing command patterns...\n");

    let report = analyzer
        .analyze(project_root.as_ref().and_then(|p| p.to_str()))
        .await?;

    println!("{}", "=".repeat(60));
    println!("📊 Analysis Report");
    println!("{}", "=".repeat(60));
    println!("\nPatterns Found: {}", report.patterns_found);
    println!("Suggestions Generated: {}", report.suggestions_generated);

    if !report.patterns.is_empty() {
        println!("\n🔗 Detected Patterns:");
        for (i, pattern) in report.patterns.iter().take(5).enumerate() {
            println!(
                "\n  {}. {:?} Pattern (confidence: {:.0}%)",
                i + 1,
                pattern.pattern_type,
                pattern.confidence * 100.0
            );
            println!("     Sequence: {}", pattern.commands.join(" → "));
        }
    }

    if !report.suggestions.is_empty() {
        println!("\n💡 Smart Suggestions:");
        for (i, suggestion) in report.suggestions.iter().enumerate() {
            println!(
                "\n  {}. {} (confidence: {:.0}%)",
                i + 1,
                suggestion.command,
                suggestion.confidence * 100.0
            );
            println!("     Reason: {}", suggestion.reason);
        }
    }

    println!("\n{}", "=".repeat(60));

    Ok(())
}

async fn handle_suggest() -> Result<()> {
    let db = Arc::new(get_database().await?);
    let analyzer = Analyzer::new(db);

    println!("\n💡 Generating suggestions...\n");

    let report = analyzer.analyze(None).await?;

    if report.suggestions.is_empty() {
        println!("No suggestions available yet.");
        println!("Use berri-recall more to build up command history!");
    } else {
        println!("{}", "=".repeat(60));
        println!("Smart Suggestions");
        println!("{}", "=".repeat(60));

        for (i, suggestion) in report.suggestions.iter().enumerate() {
            println!(
                "\n{}. {} (confidence: {:.0}%)",
                i + 1,
                suggestion.command,
                suggestion.confidence * 100.0
            );
            println!("   💭 {}", suggestion.reason);
        }

        println!("\n{}", "=".repeat(60));
        println!("\nTip: Run these commands or ignore them - recall learns from your choices!");
    }

    Ok(())
}

async fn get_database() -> Result<Database> {
    let home = dirs::home_dir().expect("Could not find home directory");
    let db_path = home.join(".berri-recall").join("commands.db");
    Database::new(db_path).await
}

fn print_usage() {
    println!(
        r#"berri-recall v{} - Your terminal remembers everything

USAGE:
    berri-recall <COMMAND> [OPTIONS]

COMMANDS:
    record <command>       Record a command
    recent [limit]         Show recent commands (default: 10)
    search <query>         Search for commands
    setup [--all]          Install shell hooks
    uninstall              Remove shell hooks
    status                 Show status and stats
    analyze                Analyze command patterns
    suggest                Get smart suggestions
    version                Show version
    help                   Show this help

EXAMPLES:
    berri-recall record npm test
    berri-recall recent 20
    berri-recall search docker
    berri-recall setup
    berri-recall status

AUTOMATIC RECORDING:
    Run 'berri-recall setup' to automatically record all commands.

For more info: https://github.com/monishobaid/berri-recall
"#,
        env!("CARGO_PKG_VERSION")
    );
}
