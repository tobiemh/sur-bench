use std::thread::sleep;
use std::time::Duration;

use clap::{Parser, ValueEnum};

use crate::benchmark::{Benchmark, DryClientProvider};
use crate::docker::{Arguments, DockerContainer};
use crate::postgres::PostgresClientProvider;
use crate::surrealdb::SurrealDBClientProvider;

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
        let (prev, after) = match self {
            Database::Dry => todo!(),
            Database::SurrealDB => {
                (Some(Arguments::new(["-p", "127.0.0.1:8000:8000"])),
                 Some(Arguments::new(["start", "--auth", "--user", "root", "--pass", "root", "memory"])))
            }
            Database::MongoDB => todo!(),
            Database::Postgresql => {
                (Some(Arguments::new(["-p", "127.0.0.1:5432:5432", "-e", "POSTGRES_PASSWORD=postgres"])),
                 None)
            }
        };
        let container = DockerContainer::start(image, prev, after);
        sleep(Duration::from_secs(10));
        container
    }

    fn run(&self, benchmark: &Benchmark) {
        match self {
            Database::Dry => benchmark.run(DryClientProvider::default()),
            Database::SurrealDB => benchmark.run(SurrealDBClientProvider::default()),
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