use postgres::{Client, NoTls};

use crate::benchmark::{BenchmarkClient, BenchmarkClientProvider, Record};

#[derive(Default)]
pub(crate) struct PostgresClientProvider {}

impl BenchmarkClientProvider<PostgresClient> for PostgresClientProvider {
    fn create_client(&self) -> PostgresClient {
        PostgresClient {
            client: Client::connect("host=localhost user=postgres password=postgres", NoTls).unwrap()
        }
    }
}

pub(crate) struct PostgresClient {
    client: Client,
}

impl BenchmarkClient for PostgresClient {
    fn prepare(&mut self) {
        self.client.batch_execute("
    CREATE TABLE record (
        id      SERIAL PRIMARY KEY,
        text    TEXT NOT NULL,
        integer    INTEGER NOT NULL
    )").unwrap();
    }

    fn write(&mut self, key: i32, record: &Record) {
        let res = self.client.execute(
            "INSERT INTO record (id, text, integer) VALUES ($1, $2, $3)",
            &[&key, &record.text, &record.integer],
        ).unwrap();
        assert_eq!(res, 1);
    }

    fn read(&mut self, key: i32) {
        let res = self.client.query("SELECT id, text, integer FROM record WHERE id=$1", &[&key]).unwrap();
        assert_eq!(res.len(), 1);
    }

    fn delete(&mut self, key: i32) {
        let res = self.client.execute(
            "DELETE FROM record WHERE id=$1",
            &[&key],
        ).unwrap();
        assert_eq!(res, 1);
    }
}