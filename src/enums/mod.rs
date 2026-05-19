use bytes::{Bytes, BytesMut};

pub enum Command {
    Append(Vec<Bytes>),
}

pub enum OffsetCommand {
    Append(BytesMut),
}

#[derive(Clone)]
pub enum AppError {
    WriteError(String),
}

pub enum WriterState {
    Healthy,
    Failure(String),
}