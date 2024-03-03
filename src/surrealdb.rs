use serde::Deserialize;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::sql::Thing;
use surrealdb::Surreal;

use crate::benchmark::{BenchmarkClient, BenchmarkClientProvider, Record};

#[derive(Default)]
pub(crate) struct SurrealDBClientProvider {}

impl BenchmarkClientProvider<SurrealDBClient> for SurrealDBClientProvider {
    async fn create_client(&self) -> SurrealDBClient {
        let db = Surreal::new::<Ws>("127.0.0.1:8000").await.unwrap();

        // Signin as a namespace, database, or root user
        db.signin(Root {
            username: "root",
            password: "root",
        })
            .await.unwrap();

        // Select a specific namespace / database
        db.use_ns("test").use_db("test").await.unwrap();

        SurrealDBClient {
            db
        }
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
    async fn prepare(&mut self) {}

    async fn write(&mut self, key: i32, record: &Record) {
        let created: Option<SurrealRecord> = self.db
            .create(("record", key))
            .content(record.clone())
            .await.unwrap_or_else(|err| panic!("Key: {key} - Error: {err}"));
        assert!(created.is_some())
    }

    async fn read(&mut self, key: i32) {
        let read: Option<Record> = self.db
            .select(("record", key))
            .await.unwrap();
        assert!(read.is_some())
    }

    async fn delete(&mut self, key: i32) {
        let deleted: Option<Record> = self.db.delete(("record", key)).await.unwrap();
        assert!(deleted.is_some())
    }
}