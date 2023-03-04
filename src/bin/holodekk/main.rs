use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use serde::Deserialize;

// use buildkit_llb::prelude::*;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Options {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Build a container from a Dockerfile
    Build {
        /// Path to the input file
        #[arg(long = "file", value_name = "file", required = true)]
        inputfile: PathBuf,
    }
}

#[derive(Deserialize)]
struct InstallInstruction {
    packages: Vec<String>,
}

#[derive(Deserialize)]
struct ConditionInstruction {
    test: String,
    success: Vec<Instruction>,
}

#[derive(Deserialize)]
enum Instruction {
    #[serde(rename = "install_packages")]
    Install(InstallInstruction),
    #[serde(rename = "condition")]
    Condition(ConditionInstruction),
}

#[derive(Deserialize)]
struct Manifest {
    os: String,
    deps: Vec<String>,
    builddeps: Vec<String>,
    instructions: Vec<Instruction>,
}

fn main() {
    let options = Options::parse();

    match &options.command {
        Commands::Build { inputfile } => {
            let file = File::open(inputfile).unwrap();
            let reader = BufReader::new(file);
            let manifest: Manifest = serde_json::from_reader(reader).unwrap();
            println!("os: {}", manifest.os);
            if !manifest.deps.is_empty() {
                println!("DEPS:");
                for dep in manifest.deps.iter() {
                    println!("  {}", dep);
                }
            }

            if !manifest.builddeps.is_empty() {
                println!("Build Deps:");
                for dep in manifest.builddeps.iter() {
                    println!("  {}", dep);
                }
            }

            for ins in manifest.instructions.iter() {
                match ins {
                    Instruction::Install(i) => {
                        println!("install instruction");
                    },
                    Instruction::Condition(c) => {
                        println!("condition instruction");
                    }
                }
            }
        },
    };
}
