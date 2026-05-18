use bytes::{Bytes, BytesMut};

pub enum Command {
    Append(Vec<Bytes>),
}

pub enum OffsetCommand {
    Append(BytesMut),
}

pub enum AppError {
    WriteError(String),
}
