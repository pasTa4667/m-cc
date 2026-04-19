use bytes::Bytes;

pub enum Command {
    Append(Vec<Bytes>),
}
