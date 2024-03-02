use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use clap::{Parser, ValueEnum};
use parking_lot::RwLock;

use crate::benchmark::{Benchmark, DryDatabase, DrySampler};
use crate::docker::DockerContainer;

mod docker;
mod benchmark;

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
    pub(crate) samples: usize,

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
    fn run_writes(&self, benchmark: &Benchmark, dry_db: DryDatabase) -> Duration {
        match self {
            Database::Dry => {
                benchmark.run_writes(DrySampler::new(dry_db))
            }
            Database::SurrealDB => todo!(),
            Database::MongoDB => todo!(),
            Database::Postgresql => todo!(),
        }
    }

    fn run_reads(&self, benchmark: &Benchmark, dry_db: DryDatabase) -> Duration {
        match self {
            Database::Dry => {
                benchmark.run_reads(DrySampler::new(dry_db))
            }
            Database::SurrealDB => todo!(),
            Database::MongoDB => todo!(),
            Database::Postgresql => todo!(),
        }
    }

    fn run_deletes(&self, benchmark: &Benchmark, dry_db: DryDatabase) -> Duration {
        match self {
            Database::Dry => {
                benchmark.run_deletes(DrySampler::new(dry_db))
            }
            Database::SurrealDB => todo!(),
            Database::MongoDB => todo!(),
            Database::Postgresql => todo!(),
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
    let mut container = args.image.map(|i| DockerContainer::start(&i));

    // Create the in memory dry db if required
    let dry_db = Arc::new(RwLock::new(HashMap::new()));

    // Run the write benchmark
    println!("Writes: {:?}", args.database.run_writes(&benchmark, dry_db.clone()));

    // Run the read benchmark
    println!("Reads: {:?}", args.database.run_reads(&benchmark, dry_db.clone()));

    // Run the read benchmark
    println!("Deletes: {:?}", args.database.run_deletes(&benchmark, dry_db.clone()));

    // Stop the container
    if let Some(mut container) = container.take() {
        container.stop();
    }
}