use anyhow::Result;
use clap::{Parser, Subcommand};
use duct::cmd;
use std::fs::{create_dir_all, remove_dir_all};
use xtaskops::ops::{clean_files, confirm};

/// Run coverage. Modification of xtaskops
/// to use --release flag
///
/// # Errors
/// Fails if any command fails
///
pub fn coverage(devmode: bool) -> Result<()> {
    create_dir_all("coverage")?;
    remove_dir_all("coverage")?;
    create_dir_all("coverage")?;

    println!("=== running coverage ===");
    cmd!("cargo", "test", "--release")
        .env("CARGO_INCREMENTAL", "0")
        .env("RUSTFLAGS", "-Cinstrument-coverage")
        .env("LLVM_PROFILE_FILE", "cargo-test-%p-%m.profraw")
        .run()?;
    println!("ok.");

    println!("=== generating report ===");
    let (fmt, file) = if devmode {
        ("html", "coverage/html")
    } else {
        ("lcov", "coverage/tests.lcov")
    };
    cmd!(
        "grcov",
        ".",
        "--binary-path",
        "./target/debug/deps",
        "-s",
        ".",
        "-t",
        fmt,
        "--branch",
        "--ignore-not-existing",
        "--ignore",
        "../*",
        "--ignore",
        "/*",
        "--ignore",
        "xtask/*",
        "--ignore",
        "*/src/tests/*",
        "-o",
        file,
    )
    .run()?;
    println!("ok.");

    println!("=== cleaning up ===");
    clean_files("**/*.profraw")?;
    println!("ok.");
    if devmode {
        if confirm("open report folder?") {
            cmd!("open", file).run()?;
        } else {
            println!("report location: {file}");
        }
    }

    Ok(())
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    subcommand: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run coverage
    Coverage {
        /// Run in dev mode
        #[clap(long)]
        devmode: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.subcommand {
        Commands::Coverage { devmode } => {
            coverage(devmode).unwrap();
        }
    }
}
