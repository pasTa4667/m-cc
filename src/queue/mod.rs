use bytes::Bytes;

pub mod in_memory;

pub trait MessageQueue: Send + Sync + 'static {
    fn push(&self, msg: Bytes) -> Result<(), ()>;
    fn pop_batch(&self, max: usize) -> Vec<Bytes>;
    fn push_batch(&self, msgs: Vec<Bytes>) -> usize;
}

pub trait ParallelQueue: Send + Sync + 'static {
    fn pop_batch_parallel(
        &self,
        max: usize,
    ) -> impl std::future::Future<Output = Vec<Bytes>> + Send;
}

pub trait AppQueue: MessageQueue + ParallelQueue {}
