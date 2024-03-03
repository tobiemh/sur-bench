use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};
use std::time::{Duration, Instant};

use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;
use rayon::scope;
use serde::{Deserialize, Serialize};
use tokio::runtime::{Builder, Runtime};
use tokio::sync::RwLock;

use crate::Args;

pub(crate) struct Benchmark {
    threads: usize,
    samples: i32,
}

impl Benchmark {
    pub(crate) fn new(args: &Args) -> Self {
        Self {
            threads: args.threads,
            samples: args.samples,
        }
    }

    pub(crate) fn run<C, P>(&self, client_provider: P) where C: BenchmarkClient + Send, P: BenchmarkClientProvider<C> + Send + Sync {
        { // Prepare
            let runtime = Runtime::new().expect("Failed to create a runtime");
            runtime.block_on(async {
                let mut client = client_provider.create_client().await;
                client.prepare().await;
            });
        }

        // Run the write benchmark
        println!("Writes: {:?}", self.run_operation(&client_provider, BenchmarkOperation::WRITE));

        // Run the read benchmark
        println!("Reads: {:?}", self.run_operation(&client_provider, BenchmarkOperation::READ));

        // Run the read benchmark
        println!("Deletes: {:?}", self.run_operation(&client_provider, BenchmarkOperation::DELETE));
    }

    fn run_operation<C, P>(&self, client_provider: &P, operation: BenchmarkOperation) -> Duration
        where C: BenchmarkClient + Send, P: BenchmarkClientProvider<C> + Send + Sync {
        let time = Instant::now();
        scope(|s| {
            let current = Arc::new(AtomicI32::new(0));
            for _ in 0..self.threads {
                let current = current.clone();
                s.spawn(move |_| {
                    let mut record_provider = RecordProvider::default();
                    let runtime = Builder::new_multi_thread()
                        .worker_threads(4) // Set the number of worker threads
                        .enable_all() // Enables all runtime features, including I/O and time
                        .build()
                        .expect("Failed to create a runtime");
                    runtime.block_on(async {
                        let mut client = client_provider.create_client().await;
                        loop {
                            let sample = current.fetch_add(1, Ordering::Relaxed);
                            if sample >= self.samples {
                                break;
                            }
                            match operation {
                                BenchmarkOperation::WRITE => {
                                    let record = record_provider.sample();
                                    client.write(sample, record).await;
                                }
                                BenchmarkOperation::READ => client.read(sample).await,
                                BenchmarkOperation::DELETE => client.delete(sample).await
                            }
                        }
                    });
                })
            }
        });
        time.elapsed()
    }
}

#[derive(Clone, Copy)]
pub(crate) enum BenchmarkOperation {
    WRITE,
    READ,
    DELETE,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub(crate) struct Record {
    pub(crate) text: String,
    pub(crate) integer: i32,
}

struct RecordProvider {
    rng: SmallRng,
    record: Record,
}

impl Default for RecordProvider {
    fn default() -> Self {
        Self {
            rng: SmallRng::from_entropy(),
            record: Default::default(),
        }
    }
}

const CHARSET: &[u8; 37] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ 0123456789";

impl RecordProvider {
    fn sample(&mut self) -> &Record {
        self.record.text = (0..50)
            .map(|_| {
                let idx = self.rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();
        &self.record
    }
}

pub(crate) trait BenchmarkClientProvider<C> where C: BenchmarkClient {
    async fn create_client(&self) -> C;
}

pub(crate) trait BenchmarkClient {
    async fn prepare(&mut self);
    async fn write(&mut self, key: i32, record: &Record);
    async fn read(&mut self, key: i32);
    async fn delete(&mut self, key: i32);
}


pub(crate) type DryDatabase = Arc<RwLock<HashMap<i32, Record>>>;

#[derive(Default)]
pub(crate) struct DryClientProvider {
    database: DryDatabase,
}

impl BenchmarkClientProvider<DryClient> for DryClientProvider {
    async fn create_client(&self) -> DryClient {
        DryClient { database: self.database.clone() }
    }
}

pub(crate) struct DryClient {
    database: DryDatabase,
}

impl BenchmarkClient for DryClient {
    async fn prepare(&mut self) {}

    async fn write(&mut self, sample: i32, record: &Record) {
        self.database.write().await.insert(sample, record.clone());
    }

    async fn read(&mut self, sample: i32) {
        assert!(self.database.read().await.get(&sample).is_some());
    }

    async fn delete(&mut self, sample: i32) {
        assert!(self.database.write().await.remove(&sample).is_some());
    }
}

