use crate::api::{
    labels,
    metrics::{Descriptor, Result},
};
use crate::sdk::{
    export::metrics::{AggregationSelector, Aggregator, CheckpointSet, Integrator, Record},
    Resource,
};
use dashmap::DashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// TODO
pub fn simple(
    selector: Box<dyn AggregationSelector + Send + Sync>,
    stateful: bool,
) -> SimpleIntegrator {
    SimpleIntegrator {
        aggregation_selector: selector,
        stateful,
        batch: DashMap::default(),
        // inner: RwLock::new(SimpleIntegratorInner {
        //     batch: HashMap::default(),
        // }),
    }
}

/// TODO
#[derive(Debug)]
pub struct SimpleIntegrator {
    aggregation_selector: Box<dyn AggregationSelector + Send + Sync>,
    stateful: bool,
    batch: DashMap<BatchKey, BatchValue>,
    // inner: RwLock<SimpleIntegratorInner>,
}

impl SimpleIntegrator {
    /// TODO
    pub fn checkpoint_set(&self) -> Iter {
        self.into_iter()
    }
    /// TODO
    pub fn finished_collection(&self) {
        if !self.stateful {
            println!("CLEARING BATCH");
            self.batch.clear();
        }
    }
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
}

// #[derive(Debug)]
// pub struct SimpleIntegratorInner {
//     batch: HashMap<BatchKey, BatchValue>,
// }
//
// impl SimpleIntegratorInner {
//     pub fn checkpoint_set(&self) -> Iter {
//         self.into_iter()
//     }
// }

impl<'a> IntoIterator for &'a SimpleIntegrator {
    type Item = dashmap::ElementGuard<BatchKey, BatchValue>;
    type IntoIter = Iter;

    fn into_iter(self) -> Self::IntoIter {
        Iter(self.batch.iter())
    }
}
/// An iterator over the entries of a `SimpleIntegratorInner`.
#[allow(missing_debug_implementations)]
pub struct Iter(dashmap::Iter<BatchKey, BatchValue>);
impl Iterator for Iter {
    type Item = dashmap::ElementGuard<BatchKey, BatchValue>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
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

impl CheckpointSet for Iter {
    fn try_for_each(&mut self, f: &mut dyn FnMut(&Record) -> Result<()>) -> Result<()> {
        Iterator::try_for_each(self, |guard| {
            f(&Record::new(
                &guard.descriptor,
                &guard.labels,
                &guard.resource,
                &guard.aggregator,
            ))
        })
    }
}

impl Integrator for SimpleIntegrator {
    fn aggregation_selector(&self) -> &dyn AggregationSelector {
        self.aggregation_selector.as_ref()
    }

    fn process(&self, record: Record) -> Result<()> {
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
            self.batch.len()
        );
        if let Some(value) = self.batch.get(&key) {
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
        if self.stateful {
            // Note: the call to AggregatorFor() followed by Merge
            // is effectively a Clone() operation.
            new_agg = self.aggregation_selector().aggregator_for(desc);
            if let Some(new_agg) = new_agg.as_ref() {
                println!("MERGING STATEFUL: {:?}", desc);
                if let Err(err) = new_agg.merge(agg, desc) {
                    return Err(err);
                }
            }
        }

        println!("INSERTING INTO INTEGRATOR BATCH: {:?}", desc);
        self.batch.insert(
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
}

/// TODO
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct BatchKey(u64);

/// TODO
#[derive(Debug)]
pub struct BatchValue {
    aggregator: Arc<dyn Aggregator + Send + Sync>,
    descriptor: Descriptor,
    labels: labels::Set,
    resource: Resource,
}
