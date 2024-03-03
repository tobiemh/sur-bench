use crate::benchmark::{BenchmarkClient, BenchmarkClientProvider, Record};

#[derive(Default)]
pub(crate) struct SurrealDBClientProvider {}

impl BenchmarkClientProvider<SurrealDBClient> for SurrealDBClientProvider {
    fn create_client(&self) -> SurrealDBClient {
        todo!()
    }
}

pub(crate) struct SurrealDBClient {}

impl BenchmarkClient for SurrealDBClient {
    fn prepare(&mut self) {
        todo!()
    }

    fn write(&mut self, key: i32, record: &Record) {
        todo!()
    }

    fn read(&mut self, key: i32) {
        todo!()
    }

    fn delete(&mut self, key: i32) {
        todo!()
    }
}