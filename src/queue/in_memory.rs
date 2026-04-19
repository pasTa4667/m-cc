use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use bytes::Bytes;
use crossbeam::queue::ArrayQueue;
use tokio::task;

use crate::queue::{MessageQueue, ParallelQueue};

pub struct ShardedQueue {
    queues: Vec<ArrayQueue<Bytes>>,
    parallel_queues: Vec<Arc<ArrayQueue<Bytes>>>,
    counter: AtomicUsize,
    pop_counter: AtomicUsize,
}

impl ShardedQueue {
    pub fn new(num_shards: usize, capacity: usize) -> Self {
        let queues = (0..num_shards).map(|_| ArrayQueue::new(capacity)).collect();
        let parallel_queues = (0..num_shards)
            .map(|_| Arc::new(ArrayQueue::new(capacity)))
            .collect();

        Self {
            queues,
            parallel_queues,
            counter: AtomicUsize::new(0),
            pop_counter: AtomicUsize::new(0),
        }
    }

    pub fn next_shard(&self) -> usize {
        self.counter.fetch_add(1, Ordering::Relaxed) % self.queues.len()
    }
}

impl MessageQueue for ShardedQueue {
    fn push(&self, msg: Bytes) -> Result<(), ()> {
        let idx = self.next_shard();
        self.queues[idx].push(msg).map_err(|_| ())
    }

    fn pop_batch(&self, max: usize) -> Vec<Bytes> {
        let mut batch = Vec::with_capacity(max);
        let start = self.pop_counter.fetch_add(1, Ordering::Relaxed);

        let len = self.queues.len();

        for i in 0..len {
            let idx = (start + i) % len;
            let q = &self.queues[idx];

            while batch.len() < max {
                match q.pop() {
                    Some(msg) => batch.push(msg),
                    None => break,
                }
            }

            if batch.len() >= max {
                break;
            }
        }

        batch
    }

    fn push_batch(&self, msgs: Vec<Bytes>) -> usize {
        let mut pushed = 0;

        for msg in msgs {
            let idx = self.next_shard();

            if self.parallel_queues[idx].push(msg).is_ok() {
                pushed += 1;
            } else {
                break;
            }
        }

        pushed
    }
}

impl ParallelQueue for ShardedQueue {
    async fn pop_batch_parallel(&self, max: usize) -> Vec<Bytes> {
        let num_shards = self.queues.len();
        let per_shard = (max / num_shards).max(1);

        let mut handles = Vec::with_capacity(num_shards);

        for q in &self.parallel_queues {
            let q = Arc::clone(q);

            handles.push(task::spawn(async move {
                let mut msgs = Vec::with_capacity(per_shard);

                for _ in 0..per_shard {
                    match q.pop() {
                        Some(msg) => msgs.push(msg),
                        None => break,
                    }
                }

                msgs
            }));
        }

        let mut batch = Vec::with_capacity(max);

        for h in handles {
            if let Ok(mut msgs) = h.await {
                batch.append(&mut msgs);
            }
        }

        batch
    }
}
