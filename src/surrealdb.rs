use anyhow::Result;
use serde::Deserialize;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::sql::Thing;
use surrealdb::Surreal;

use crate::benchmark::{BenchmarkClient, BenchmarkClientProvider, Record};
use crate::docker::DockerParams;

pub(crate) const SURREAL_SPEEDB_DOCKER_PARAMS: DockerParams = DockerParams {
	image: "surrealdb/surrealdb:v1.2.1",
	pre_args: "-p 127.0.0.1:8000:8000",
	post_args: "start --auth --user root --pass root speedb://tmp/sur-bench.db",
};

pub(crate) const SURREAL_ROCKSDB_DOCKER_PARAMS: DockerParams = DockerParams {
	image: "surrealdb/surrealdb:v1.2.1",
	pre_args: "-p 127.0.0.1:8000:8000",
	post_args: "start --auth --user root --pass root rocksdb://tmp/sur-bench.db",
};

pub(crate) const SURREAL_MEMORY_DOCKER_PARAMS: DockerParams = DockerParams {
	image: "surrealdb/surrealdb:v1.2.1",
	pre_args: "-p 127.0.0.1:8000:8000",
	post_args: "start --auth --user root --pass root memory",
};

#[derive(Default)]
pub(crate) struct SurrealDBClientProvider {}

impl BenchmarkClientProvider<SurrealDBClient> for SurrealDBClientProvider {
	async fn create_client(&self) -> Result<SurrealDBClient> {
		let db = Surreal::new::<Ws>("127.0.0.1:8000").await?;

		// Signin as a namespace, database, or root user
		db.signin(Root {
			username: "root",
			password: "root",
		})
		.await?;

		// Select a specific namespace / database
		db.use_ns("test").use_db("test").await?;

		Ok(SurrealDBClient {
			db,
		})
	}
}

pub(crate) struct SurrealDBClient {
	db: Surreal<Client>,
}

#[derive(Debug, Deserialize)]
struct SurrealRecord {
	#[allow(dead_code)]
	id: Thing,
}

impl BenchmarkClient for SurrealDBClient {
	async fn prepare(&mut self) -> Result<()> {
		Ok(())
	}

	async fn write(&mut self, key: i32, record: &Record) -> Result<()> {
		let created: Option<SurrealRecord> =
			self.db.create(("record", key)).content(record.clone()).await?;
		assert!(created.is_some());
		Ok(())
	}

	async fn read(&mut self, key: i32) -> Result<()> {
		let read: Option<Record> = self.db.select(("record", key)).await?;
		assert!(read.is_some());
		Ok(())
	}

	async fn delete(&mut self, key: i32) -> Result<()> {
		let deleted: Option<Record> = self.db.delete(("record", key)).await?;
		assert!(deleted.is_some());
		Ok(())
	}
}
