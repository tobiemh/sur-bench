use std::thread::sleep;
use std::time::Duration;

use clap::{Parser, ValueEnum};

use crate::benchmark::{Benchmark, DryClientProvider};
use crate::docker::{Arguments, DockerContainer};
use crate::postgres::PostgresClientProvider;

mod docker;
mod benchmark;
mod postgres;
mod surrealdb;

#[derive(Parser, Debug)]
#[command(term_width = 0)]
pub(crate) struct Args {
    /// Docker image
    #[arg(short, long)]
    pub(crate) image: Option<String>,

    /// Database
    #[arg(short, long)]
    pub(crate) database: Database,

    /// Number of samples
    #[clap(short, long)]
    pub(crate) samples: i32,

    /// Number of concurrent threads
    #[clap(short, long)]
    pub(crate) threads: usize,
}


#[derive(ValueEnum, Debug, Clone)]
pub(crate) enum Database {
    Dry,
    SurrealDB,
    MongoDB,
    Postgresql,
}

impl Database {
    fn start_docker(&self, image: &str) -> DockerContainer {
        let args = match self {
            Database::Dry => todo!(),
            Database::SurrealDB => todo!(),
            Database::MongoDB => todo!(),
            Database::Postgresql => {
                Arguments::new(["-p", "127.0.0.1:5432:5432", "-e", "POSTGRES_PASSWORD=postgres"])
            }
        };
        let container = DockerContainer::start(image, args);
        sleep(Duration::from_secs(5));
        container
    }

    fn run(&self, benchmark: &Benchmark) {
        match self {
            Database::Dry => benchmark.run(DryClientProvider::default()),
            Database::SurrealDB => todo!(),
            Database::MongoDB => todo!(),
            Database::Postgresql => benchmark.run(PostgresClientProvider::default()),
        }
    }
}

fn main() {
    let args = Args::parse();
    println!("Image: {:?}", args.image);
    println!("Database: {:?}", args.database);
    println!("Samples: {:?}", args.samples);
    println!("Threads: {:?}", args.threads);

    // Prepare the benchmark
    let benchmark = Benchmark::new(&args);

    // Spawn the docker image if any
    let mut container = args.image.map(|i| args.database.start_docker(&i));

    args.database.run(&benchmark);

    // Stop the container
    if let Some(mut container) = container.take() {
        container.stop();
    }
}