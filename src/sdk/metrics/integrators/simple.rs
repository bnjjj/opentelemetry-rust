use crate::api::{
    labels,
    metrics::{Descriptor, Result},
};
use crate::sdk::{
    export::metrics::{
        AggregationSelector, Aggregator, CheckpointSet, Integrator, LockedIntegrator, Record,
    },
    Resource,
};
// use dashmap::DashMap;
use std::collections::{hash_map::DefaultHasher, HashMap};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, MutexGuard};

/// TODO
pub fn simple(
    selector: Box<dyn AggregationSelector + Send + Sync>,
    stateful: bool,
) -> SimpleIntegrator {
    SimpleIntegrator {
        aggregation_selector: selector,
        stateful,
        batch: Mutex::new(SimpleIntegratorBatch::default()),
    }
}

/// TODO
#[derive(Debug)]
pub struct SimpleIntegrator {
    aggregation_selector: Box<dyn AggregationSelector + Send + Sync>,
    stateful: bool,
    batch: Mutex<SimpleIntegratorBatch>,
}

impl SimpleIntegrator {
    // TODO
    // pub fn checkpoint_set(&self) -> Iter {
    //     self.into_iter()
    // }
    // TODO
    // pub fn finished_collection(&self) {
    //     if !self.stateful {
    //         println!("CLEARING BATCH");
    //         // self.batch.clear();
    //     }
    // }
    // /// TODO
    // pub fn write<F, T>(&self, mut f: F) -> Result<T>
    // where
    //     F: FnMut(&mut SimpleIntegratorInner) -> Result<T>,
    // {
    //     println!("SimpleIntegrator write locking");
    //     self.inner
    //         .write()
    //         .map_err(|lock_err| MetricsError::Other(lock_err.to_string()))
    //         .and_then(|mut inner| {
    //             println!("write lock success");
    //             f(&mut inner)
    //         })
    // }
    /// TODO
    pub fn lock(&self) -> Result<SimpleLockedIntegrator<'_>> {
        self.batch
            .try_lock()
            .map_err(From::from)
            .map(move |mut locked| SimpleLockedIntegrator {
                parent: self,
                batch: locked,
            })
    }
}

impl Integrator for SimpleIntegrator {
    fn aggregation_selector(&self) -> &dyn AggregationSelector {
        self.aggregation_selector.as_ref()
    }
}

///TODO
#[derive(Debug)]
pub struct SimpleLockedIntegrator<'a> {
    parent: &'a SimpleIntegrator,
    batch: MutexGuard<'a, SimpleIntegratorBatch>,
}

impl<'a> LockedIntegrator for SimpleLockedIntegrator<'a> {
    fn process(&mut self, record: Record) -> Result<()> {
        let desc = record.descriptor();
        let mut hasher = DefaultHasher::new();
        desc.hash(&mut hasher);
        record.labels().equivalent().hash(&mut hasher);
        // FIXME: convert resource to use labels::Set
        // record.resource().equivalent().hash(&mut hasher);
        let key = BatchKey(hasher.finish());
        let agg = record.aggregator();
        let mut new_agg = None;
        println!(
            "processing record, existing batch len: {}",
            self.batch.0.len()
        );
        if let Some(value) = self.batch.0.get(&key) {
            // Note: The call to Merge here combines only
            // identical records.  It is required even for a
            // stateless Integrator because such identical records
            // may arise in the Meter implementation due to race
            // conditions.
            dbg!("MERGING", desc);
            return value.aggregator.merge(agg, desc);
        } else {
            println!("No batch entry for: {:?}", desc);
        }
        // If this integrator is stateful, create a copy of the
        // Aggregator for long-term storage.  Otherwise the
        // Meter implementation will checkpoint the aggregator
        // again, overwriting the long-lived state.
        if self.parent.stateful {
            // Note: the call to AggregatorFor() followed by Merge
            // is effectively a Clone() operation.
            new_agg = self.parent.aggregation_selector().aggregator_for(desc);
            if let Some(new_agg) = new_agg.as_ref() {
                println!("MERGING STATEFUL: {:?}", desc);
                if let Err(err) = new_agg.merge(agg, desc) {
                    return Err(err);
                }
            }
        }

        println!("INSERTING INTO INTEGRATOR BATCH: {:?}", desc);
        self.batch.0.insert(
            key,
            BatchValue {
                // FIXME consider perf of all this
                aggregator: new_agg.unwrap_or_else(|| agg.clone()),
                descriptor: desc.clone(),
                labels: record.labels().clone(),
                resource: record.resource().clone(),
            },
        );

        Ok(())
    }

    fn checkpoint_set(&mut self) -> &mut dyn CheckpointSet {
        &mut *self.batch
    }

    fn finished_collection(&mut self) {
        if !self.parent.stateful {
            println!("CLEARING BATCH");
            self.batch.0.clear();
        }
    }
}

#[derive(Debug, Default)]
struct SimpleIntegratorBatch(HashMap<BatchKey, BatchValue>);

impl CheckpointSet for SimpleIntegratorBatch {
    fn try_for_each(&mut self, f: &mut dyn FnMut(&Record) -> Result<()>) -> Result<()> {
        self.0.iter().try_for_each(|(_key, value)| {
            f(&Record::new(
                &value.descriptor,
                &value.labels,
                &value.resource,
                &value.aggregator,
            ))
        })
    }
}

// fn process_record<'a>(
//     guard: dashmap::ElementGuard<BatchKey, BatchValue>,
//     f: &dyn Fn(Record<'a>) -> Result<()>,
// ) -> Result<()> {
//     f(Record::new(
//         &guard.descriptor,
//         &guard.labels,
//         &guard.resource,
//         &guard.aggregator,
//     ))
// }

/// TODO
#[derive(Debug, PartialEq, Eq, Hash)]
struct BatchKey(u64);

/// TODO
#[derive(Debug)]
struct BatchValue {
    aggregator: Arc<dyn Aggregator + Send + Sync>,
    descriptor: Descriptor,
    labels: labels::Set,
    resource: Resource,
}
