use std::path::{Path, PathBuf};
// use buildkit_llb::prelude::*;

use clap::{Parser, Subcommand};

// use serde::Deserialize;

use flate2::write::GzEncoder;
use flate2::Compression;

use hyper::{Body, Client, Method, Request, Uri as HyperUri};
use hyper::body::HttpBody as _;
use hyperlocal::{UnixConnector, Uri};

use serde::Deserialize;

use tar::Builder as TarBuilder;

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
        /// Path to the build context
        #[arg(long = "context", value_name = "dir", required = true)]
        context: PathBuf,

        /// Compress the context before sending
        #[arg(long)]
        compress: bool
    }
}

// #[derive(Deserialize)]
// struct InstallInstruction {
//     packages: Vec<String>,
// }

// #[derive(Deserialize)]
// struct ConditionInstruction {
//     test: String,
//     success: Vec<Instruction>,
// }

// #[derive(Deserialize)]
// enum Instruction {
//     #[serde(rename = "install_packages")]
//     Install(InstallInstruction),
//     #[serde(rename = "condition")]
//     Condition(ConditionInstruction),
// }

// #[derive(Deserialize)]
// struct Manifest {
//     os: String,
//     deps: Vec<String>,
//     builddeps: Vec<String>,
//     instructions: Vec<Instruction>,
// }

// fn parse_manifest(file: File) {
//     let reader = BufReader::new(file);
//     let manifest: Manifest = serde_json::from_reader(reader).unwrap();
//     println!("os: {}", manifest.os);
//     if !manifest.deps.is_empty() {
//         println!("DEPS:");
//         for dep in manifest.deps.iter() {
//             println!("  {}", dep);
//         }
//     }

//     if !manifest.builddeps.is_empty() {
//         println!("Build Deps:");
//         for dep in manifest.builddeps.iter() {
//             println!("  {}", dep);
//         }
//     }

//     for ins in manifest.instructions.iter() {
//         match ins {
//             Instruction::Install(i) => {
//                 println!("install instruction");
//             },
//             Instruction::Condition(c) => {
//                 println!("condition instruction");
//             }
//         }
//     }
// }

#[derive(Deserialize)]
struct AuxResponse {
    #[serde(rename = "ID")]
    id: String,
}


#[derive(Deserialize)]
enum BuildResponse {
    #[serde(rename = "stream")]
    Stream(String),
    #[serde(rename = "aux")]
    Aux(AuxResponse),
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let options = Options::parse();

    match &options.command {
        Commands::Build { context, compress } => {
            build(&context, compress).await?;
            // let file = File::open(inputfile).unwrap();
            // build(&manifest);
        },
    };

    Ok(())
}

fn create_archive<T: std::io::Write, P: AsRef<Path>>(context: P, target: T) -> std::io::Result<()> {
    let mut archive: TarBuilder<T> = TarBuilder::new(target);
    archive.append_dir_all("", context.as_ref())
}

async fn build<F: AsRef<Path>>(context: F, compress: &bool) -> std::io::Result<()> {
    // create a tar archive
    println!("generating archive");
    let mut bytes = Vec::default();
    if *compress {
        let enc = GzEncoder::new(&mut bytes, Compression::default());
        create_archive(context, enc)?;
    } else {
        create_archive(context, &mut bytes)?;
    }

    // post the tar to the server
    println!("Sending build context to Holodekk");
    println!("Size: {}", bytes.len());
    let connector = UnixConnector;
    let uri: HyperUri = Uri::new("/var/run/holodekk.sock", "/build").into();
    let client: Client<UnixConnector, Body> = Client::builder().build(connector);
    let docker_req = Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header("Content-Type", "application/x-tar")
        .body(Body::from(bytes))
        .expect("request builder");

    let resp = client.request(docker_req).await;

    match resp {
        Ok(mut r) => {
            while let Some(next) = r.data().await {
                match next {
                    Err(e) => {
                        println!("error getting data: {:?}", e);
                    },
                    Ok(data) => {
                        let line = std::str::from_utf8(&data).unwrap();
                        let line: BuildResponse = serde_json::from_str(line).unwrap();
                        match line {
                            BuildResponse::Stream(msg) => { print!("{}", msg); },
                            BuildResponse::Aux(aux) => { print!("built image: {}" , aux.id); },
                        }
                    },
                }
            }
        },
        Err(err) => {
            println!("Error: {}", err);
        }
    }
    // display the results
    Ok(())
}

// fn build_manifest(manifest: &Manifest) {
//     let builder_image =
//         Source::image("library/alpine:latest").custom_name("Using alpine:latest as a builder");

//     let command = {
//         Command::run("/bin/sh")
//             .args(&["-c", "echo 'test string 5' > /out/file0"])
//             .custom_name("create a dummy file")
//             .mount(Mount::ReadOnlyLayer(builder_image.output(), "/"))
//             .mount(Mount::Scratch(OutputIdx(0), "/out"))
//     };

//     let fs = {
//         FileSystem::sequence()
//             .custom_name("do multiple file system manipulations")
//             .append(
//                 FileSystem::copy()
//                     .from(LayerPath::Other(command.output(0), "/file0"))
//                     .to(OutputIdx(0), LayerPath::Other(command.output(0), "/file1")),
//             )
//             .append(
//                 FileSystem::copy()
//                     .from(LayerPath::Own(OwnOutputIdx(0), "/file0"))
//                     .to(OutputIdx(1), LayerPath::Own(OwnOutputIdx(0), "/file2")),
//             )
//     };

//     Terminal::with(fs.output(1))
//         .write_definition(stdout())
//         .unwrap()
// }
