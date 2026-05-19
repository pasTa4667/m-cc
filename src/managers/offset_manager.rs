use std::{
    collections::HashMap, fs::{File, OpenOptions}, io::{BufReader, BufWriter, Write}, path::PathBuf, sync::Arc, time::{Duration, Instant}
};

use bytes::{BufMut, BytesMut};
use crossbeam::channel::{Sender, unbounded};
use parking_lot::Mutex;

use crate::enums::{AppError, OffsetCommand, WriterState};

type WriterStateType = Arc<Mutex<WriterState>>;

 #[derive(Eq, Hash, PartialEq, Clone)]
pub struct OffsetKey {
    pub topic: String,
    pub group_id: Option<String>,
}

/// Offset Manager for Consumer Backup Offsets
pub struct OffsetManager {
    offsets: Mutex<HashMap<OffsetKey, usize>>,
    reader: Mutex<BufReader<File>>,
    sender: Sender<OffsetCommand>,
    writer_state: WriterStateType,
}

impl OffsetManager {
    pub fn new(path: &str) -> Self {
        let mut path_buf = PathBuf::from(path);
        path_buf.push("consumer_offsets_backup.log");

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(&path_buf)
            .unwrap();
        
        let writer_state = Arc::new(Mutex::new(WriterState::Healthy));

        let reader = Mutex::new(BufReader::new(file));

        let sender = spawn_writer(path_buf, writer_state.clone());

        Self {
            offsets: Mutex::new(HashMap::new()),
            reader,
            sender,
            writer_state,
        }
    }

    pub fn read(&self) -> usize {
        0
    }

    pub fn append(&self, offset_key: OffsetKey, offset: usize) -> Result<(), AppError> {
        let offset_to_save = offset_key.clone();

        match &*self.writer_state.lock() {
            WriterState::Healthy => {},
            WriterState::Failure(error) => {
                return Err(AppError::WriteError(error.to_owned()));
            }
        }

        let mut buf = BytesMut::new();
        match offset_to_save.group_id {
            Some(group_id) => {
                buf.put_u32(group_id.len() as u32);
                buf.put_slice(group_id.as_bytes());
            },
            None => {
                buf.put_u32(0);
            }
        }

        buf.put_u32(offset_to_save.topic.len() as u32);
        buf.put_slice(offset_to_save.topic.as_bytes());

        buf.put_u64(offset as u64);

        match self.sender
            .send(OffsetCommand::Append(buf)) {
                Ok(()) => {
                    self.insert_offset(offset_key, offset);
                    Ok(())
                },
                Err(_) => {
                    Err(AppError::WriteError("Failed to persist offset".to_string()))
                }
            }
    }

    fn insert_offset(&self, offset_key: OffsetKey, offset: usize) {
        let mut offsets = self.offsets.lock();
        offsets.insert(offset_key, offset);
    }
}

fn spawn_writer(path_buf: PathBuf, writer_state: WriterStateType) -> Sender<OffsetCommand> {
    let (tx, rx) = unbounded::<OffsetCommand>();

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .read(true)
        .open(path_buf)
        .unwrap();

    std::thread::spawn(move || {
        let mut writer = BufWriter::with_capacity(2 << 20, file);

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
                        batch_buf.extend_from_slice(&offsets);
                        bytes_written += offsets.len();
                    }
                }
            }

            if let Err(e) = writer.write_all(&batch_buf) {
                *writer_state.lock() = WriterState::Failure(e.to_string());
                break;
            }

            if bytes_written >= flush_bytes || last_flush.elapsed() >= flush_interval {
                if let Err(e) = writer.flush() {
                    *writer_state.lock() = WriterState::Failure(e.to_string());
                    break; 
                }
                bytes_written = 0;
                last_flush = Instant::now();
            }
        }

        if let Err(e) = writer.flush() {
            *writer_state.lock() = WriterState::Failure(e.to_string());
        }
    });

    tx
}
