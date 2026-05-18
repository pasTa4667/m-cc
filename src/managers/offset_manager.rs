use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter, Write},
    path::PathBuf,
    time::{Duration, Instant},
};

use bytes::{BufMut, Bytes, BytesMut};
use crossbeam::channel::{Sender, unbounded};
use parking_lot::Mutex;

use crate::enums::{AppError, OffsetCommand};

pub struct OffsetKey {
    pub topic: String,
    pub group_id: Option<String>,
}

/// Offset Manager for Consumer Backup Offsets
pub struct OffsetManager {
    offsets: HashMap<OffsetKey, usize>,
    reader: Mutex<BufReader<File>>,
    sender: Sender<OffsetCommand>,
}

impl OffsetManager {
    pub fn new(path: &str) -> Self {
        let mut path_buf = PathBuf::from(path);
        path_buf.push("/consumer_offsets_backup.txt");

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(&path_buf)
            .unwrap();

        let reader = Mutex::new(BufReader::new(file));

        let sender = spawn_writer(path_buf);

        Self {
            offsets: HashMap::new(),
            reader,
            sender,
        }
    }

    pub fn read(&self) -> usize {
        // TODO
        0
    }

    pub fn append(&self, offset_key: OffsetKey, offset: usize) -> Result<(), AppError> {
        let mut buf = BytesMut::new();
        if let Some(group_id) = offset_key.group_id {
            buf.put_u32(group_id.len() as u32);
            buf.put_slice(group_id.as_bytes());
        }

        buf.put_u32(offset_key.topic.len() as u32);
        buf.put_slice(offset_key.topic.as_bytes());

        buf.put_slice(&offset.to_be_bytes());
        buf.put_u32(offset as u32);

        let result = self.sender.send(OffsetCommand::Append(buf));

        if result.is_err() {
            return Err(AppError::WriteError("Failed to commit offset".to_string()));
        }

        Ok(())
    }
}

fn spawn_writer(path_buf: PathBuf) -> Sender<OffsetCommand> {
    let (tx, rx) = unbounded::<OffsetCommand>();

    std::thread::spawn(move || {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(path_buf)
            .unwrap();

        let mut writer = BufWriter::with_capacity(2 << 20, file); // 8MB

        let max_batch_command = 65536;
        let flush_bytes = 33554432;
        let flush_interval = Duration::from_millis(500);

        let mut buffer = Vec::with_capacity(124);
        let mut bytes_written = 0usize;
        let mut last_flush = Instant::now();

        loop {
            match rx.recv() {
                Ok(cmd) => buffer.push(cmd),
                Err(_) => break,
            };

            while buffer.len() < max_batch_command {
                match rx.try_recv() {
                    Ok(cmd) => buffer.push(cmd),
                    Err(_) => break,
                }
            }

            let mut batch_buf = Vec::with_capacity(124);

            for cmd in buffer.drain(..) {
                match cmd {
                    OffsetCommand::Append(offsets) => {
                        for ofs in offsets {
                            batch_buf.extend_from_slice(&(ofs).to_be_bytes());
                            batch_buf.push(ofs);
                            bytes_written += 4 + ofs as usize;
                        }
                    }
                }
            }

            writer.write_all(&batch_buf).unwrap();

            if bytes_written >= flush_bytes || last_flush.elapsed() >= flush_interval {
                writer.flush().unwrap();
                bytes_written = 0;
                last_flush = Instant::now();
            }
        }

        writer.flush().unwrap();
    });

    tx
}
