use tokio_postgres::{Client, NoTls};

use crate::benchmark::{BenchmarkClient, BenchmarkClientProvider, Record};

#[derive(Default)]
pub(crate) struct PostgresClientProvider {}

impl BenchmarkClientProvider<PostgresClient> for PostgresClientProvider {
    async fn create_client(&self) -> PostgresClient {
        let (client, connection) =
            tokio_postgres::connect("host=localhost user=postgres password=postgres", NoTls).await.unwrap();
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });
        PostgresClient {
            client,
        }
    }
}

pub(crate) struct PostgresClient {
    client: Client,
}

impl BenchmarkClient for PostgresClient {
    async fn prepare(&mut self) {
        self.client.batch_execute("
    CREATE TABLE record (
        id      SERIAL PRIMARY KEY,
        text    TEXT NOT NULL,
        integer    INTEGER NOT NULL
    )").await.unwrap();
    }

    async fn write(&mut self, key: i32, record: &Record) {
        let res = self.client.execute(
            "INSERT INTO record (id, text, integer) VALUES ($1, $2, $3)",
            &[&key, &record.text, &record.integer],
        ).await.unwrap();
        assert_eq!(res, 1);
    }

    async fn read(&mut self, key: i32) {
        let res = self.client.query("SELECT id, text, integer FROM record WHERE id=$1", &[&key]).await.unwrap();
        assert_eq!(res.len(), 1);
    }

    async fn delete(&mut self, key: i32) {
        let res = self.client.execute(
            "DELETE FROM record WHERE id=$1",
            &[&key],
        ).await.unwrap();
        assert_eq!(res, 1);
    }
}