use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use parking_lot::RwLock;
use rayon::scope;
use serde_json::Value;

use crate::Args;

pub(crate) struct Benchmark {
    threads: usize,
    samples: usize,
}

impl Benchmark {
    pub(crate) fn new(args: &Args) -> Self {
        Self {
            threads: args.threads,
            samples: args.samples,
        }
    }

    fn run<F>(&self, operation: F) where F: Fn(usize, &mut ValueProvider) + Sync + Send + Copy {
        scope(|s| {
            let current = Arc::new(AtomicUsize::new(0));
            for _ in 0..self.threads {
                let current = current.clone();
                s.spawn(move |_| {
                    let mut value_provider = ValueProvider::default();
                    loop {
                        let sample = current.fetch_add(1, Ordering::Relaxed);
                        if sample >= self.samples {
                            break;
                        }
                        operation(sample, &mut value_provider);
                    }
                })
            }
        });
    }

    pub(crate) fn run_writes<S>(&self, sampler: S) -> Duration where S: Sampler {
        let start = Instant::now();
        self.run(|sample, vp| sampler.write(sample, vp.sample(sample)));
        start.elapsed()
    }

    pub(crate) fn run_reads<S>(&self, sampler: S) -> Duration where S: Sampler {
        let start = Instant::now();
        self.run(|sample, _| sampler.read(sample));
        start.elapsed()
    }

    pub(crate) fn run_deletes<S>(&self, sampler: S) -> Duration where S: Sampler {
        let start = Instant::now();
        self.run(|sample, _| sampler.delete(sample));
        start.elapsed()
    }
}

struct ValueProvider {
    value: Value,
}

impl Default for ValueProvider {
    fn default() -> Self {
        Self {
            value: Value::Number(0.into()),
        }
    }
}

impl ValueProvider {
    fn sample(&mut self, sample: usize) -> &Value {
        self.value = Value::Number(sample.into());
        &self.value
    }
}

// pub(crate) trait Operation {
//     fn operate(&self, sample: usize, value: &Value);
// }
//
// pub(crate) struct WriteOperation<S> where S: Sampler {
//     sampler: S,
// }
//
// impl<S> WriteOperation<S> where S: Sampler {
//     pub(crate) fn new(sampler: S) -> Self {
//         Self { sampler }
//     }
// }
//
// impl<S> Operation for WriteOperation<S> where S: Sampler {
//     fn operate(&self, sample: usize, value: &Value) {
//         self.sampler.write(sample, value);
//     }
// }

pub(crate) trait Sampler: Sync {
    fn write(&self, key: usize, value: &Value);
    fn read(&self, key: usize);
    fn delete(&self, key: usize);
}


pub(crate) type DryDatabase = Arc<RwLock<HashMap<usize, Value>>>;

pub(crate) struct DrySampler {
    database: DryDatabase,
}


impl DrySampler {
    pub(crate) fn new(database: DryDatabase) -> Self {
        Self {
            database,
        }
    }
}

impl Sampler for DrySampler {
    fn write(&self, sample: usize, value: &Value) {
        self.database.write().insert(sample, value.clone());
    }

    fn read(&self, sample: usize) {
        assert!(self.database.read().get(&sample).is_some());
    }

    fn delete(&self, sample: usize) {
        assert!(self.database.write().remove(&sample).is_some());
    }
}

