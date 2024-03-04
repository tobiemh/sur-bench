use anyhow::Result;
use tokio_postgres::{Client, NoTls};

use crate::benchmark::{BenchmarkClient, BenchmarkClientProvider, Record};
use crate::docker::DockerParams;

pub(crate) const POSTGRES_DOCKER_PARAMS: DockerParams = DockerParams {
    image: "postgres",
    pre_args: "-p 127.0.0.1:5432:5432 -e POSTGRES_PASSWORD=postgres",
    post_args: "",
};

#[derive(Default)]
pub(crate) struct PostgresClientProvider {}

impl BenchmarkClientProvider<PostgresClient> for PostgresClientProvider {
    async fn create_client(&self) -> Result<PostgresClient> {
        let (client, connection) =
            tokio_postgres::connect("host=localhost user=postgres password=postgres", NoTls)
                .await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });
        Ok(PostgresClient {
            client,
        })
    }
}

pub(crate) struct PostgresClient {
    client: Client,
}

impl BenchmarkClient for PostgresClient {
    async fn prepare(&mut self) -> Result<()> {
        self.client
            .batch_execute(
                "
    CREATE TABLE record (
        id      SERIAL PRIMARY KEY,
        text    TEXT NOT NULL,
        integer    INTEGER NOT NULL
    )",
            )
            .await?;
        Ok(())
    }

    async fn write(&mut self, key: i32, record: &Record) -> Result<()> {
        let res = self
            .client
            .execute(
                "INSERT INTO record (id, text, integer) VALUES ($1, $2, $3)",
                &[&key, &record.text, &record.integer],
            )
            .await?;
        assert_eq!(res, 1);
        Ok(())
    }

    async fn read(&mut self, key: i32) -> Result<()> {
        let res = self
            .client
            .query("SELECT id, text, integer FROM record WHERE id=$1", &[&key])
            .await
            ?;
        assert_eq!(res.len(), 1);
        Ok(())
    }

    async fn delete(&mut self, key: i32) -> Result<()> {
        let res = self.client.execute("DELETE FROM record WHERE id=$1", &[&key]).await?;
        assert_eq!(res, 1);
        Ok(())
    }
}
