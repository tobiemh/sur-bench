use anyhow::Result;
use clap::{Parser, ValueEnum};
use log::{error, info};

use crate::benchmark::{Benchmark, DryClientProvider};
use crate::docker::DockerContainer;
use crate::postgres::{PostgresClientProvider, POSTGRES_DOCKER_PARAMS};
use crate::surrealdb::{SurrealDBClientProvider, SURREAL_DOCKER_PARAMS};

mod benchmark;
mod docker;
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
	fn start_docker(&self, image: Option<String>) -> Option<DockerContainer> {
		let params = match self {
			Database::Dry => return None,
			Database::SurrealDB => SURREAL_DOCKER_PARAMS,
			Database::MongoDB => todo!(),
			Database::Postgresql => POSTGRES_DOCKER_PARAMS,
		};
		let container = DockerContainer::start(image, &params);
		Some(container)
	}

	fn run(&self, benchmark: &Benchmark) -> Result<()> {
		match self {
			Database::Dry => benchmark.run(DryClientProvider::default()),
			Database::SurrealDB => benchmark.run(SurrealDBClientProvider::default()),
			Database::MongoDB => todo!(),
			Database::Postgresql => benchmark.run(PostgresClientProvider::default()),
		}
	}
}

fn main() {
	// Initialise the logger
	env_logger::init();
	info!("Benchmark started!");

	// Parse the command line arguments
	let args = Args::parse();

	// Prepare the benchmark
	let benchmark = Benchmark::new(&args);

	// Spawn the docker image if any
	let mut container = args.database.start_docker(args.image);

	// Run the benchmark
	if let Err(e) = args.database.run(&benchmark) {
		error!("Error: {}", e);
		if let Some(container) = &container {
			container.logs();
		}
	}

	// Stop the container (if any)
	if let Some(mut container) = container.take() {
		container.stop();
	}
}
