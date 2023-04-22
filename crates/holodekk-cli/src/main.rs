use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Options {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Project {
        /// Directory where the subroutine is located.
        #[arg(short, long, default_value = "holodekk")]
        directory: String,

        /// Name of the subroutine to run
        #[arg(default_value = "default")]
        name: String,
    },
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // let options = Options::parse();

    // Start a Holodekk
    // let holodekk_options = HolodekkConfig {
    //     fleet: "local".to_string(),
    //     root_path: "~/.holodekk".into(),
    //     bin_path: TEMPORARY_BIN.into(),
    //     api_config: ConnectionInfo::unix("~/.holodekk/holodekk.sock"),
    //     repo_kind: RepositoryKind::Memory,
    // };
    // let holodekk = Arc::new(Holodekk::new(holodekk_options));
    // holodekk.init()?;

    // match &options.command {
    //     Commands::Project { directory, name } => match runtime::detect(holodekk, directory, name) {
    //         Ok(runtime) => {
    //             runtime.project().await?;
    //         }
    //         Err(err) => match err {
    //             CliRuntimeError::ArgumentError(reason) => {
    //                 eprintln!("{}", reason);
    //             }
    //             _ => {
    //                 eprintln!("Unknown error.");
    //             }
    //         },
    //     },
    // };

    Ok(())
}
