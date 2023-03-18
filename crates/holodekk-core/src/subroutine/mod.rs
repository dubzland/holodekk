pub mod executors;

use std::fmt::{self, Display};
use std::path::PathBuf;

// use flate2::write::GzEncoder;
// use flate2::Compression;

use hyper::{Body, Client, Method, Request, Uri as HyperUri};
use hyper::body::HttpBody as _;
use hyperlocal::{UnixConnector, Uri};

use serde::{Deserialize, Serialize};

use tar::{Builder as TarBuilder, Header};

use self::executors::Executor;


#[derive(Debug, Deserialize, Serialize)]
pub struct Subroutine {
    pub name: String,
    pub executor: Executor,
}

impl Display for Subroutine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Subroutine:")?;
        writeln!(f, "{}", serde_json::to_string_pretty(self).unwrap())
    }
}

impl Subroutine {
    pub async fn build(&self, directory: &PathBuf) -> std::io::Result<()> {
        let dockerfile = "FROM ruby:3.2.1\n \
            RUN mkdir /holodekk\n \
            WORKDIR /holodekk\n \
            ADD Gemfile Gemfile.lock /holodekk/\n \
            RUN pwd && ls -l && bundle install\n \
            ADD * /holodekk/\n \
            ENTRYPOINTT /holodekk/default.rb";

        let mut bytes = Vec::default();
        create_archive(directory, dockerfile, &mut bytes)?;

        println!("Sending build context to Holodekk");
        println!("Size: {}", bytes.len());

        let connector = UnixConnector;
        let uri: HyperUri = Uri::new("/var/run/holodekk.sock", "/subroutines/build?query=1").into();
        let client: Client<UnixConnector, Body> = Client::builder().build(connector);
        let docker_req = Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header("Content-Type", "application/x-tar")
            .body(Body::from(bytes))
            .expect("request builder");

        let resp = client.request(docker_req).await;

        println!("response received");

        match resp {
            Ok(mut r) => {
                while let Some(data) = r.data().await {
                    match data {
                        Ok(string) => {
                            println!("data: {:?}", string);
                        },
                        Err(_) => {
                        }
                    }
                }
                Ok(())
            },
            Err(err) => {
                println!("Received error from holodekk: {:?}", err);
                Ok(())
            }
        }
    }
}

fn create_archive<T: std::io::Write>(context: &PathBuf, dockerfile: &str, target: T) -> std::io::Result<()> {
    println!("Generating archive.");

    let mut archive: TarBuilder<T> = TarBuilder::new(target);

    let mut header = Header::new_gnu();
    let bytes = dockerfile.as_bytes().to_vec();
    header.set_size(bytes.len().try_into().unwrap());
    header.set_cksum();

    archive.append_data(&mut header, "Dockerfile", dockerfile.as_bytes()).unwrap();

    archive.append_dir_all("", context)?;

    println!("archive created.");
    Ok(())
}
