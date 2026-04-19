use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::time::{Duration, Instant};

use bytes::Bytes;
use crossbeam::channel::{Sender, unbounded};
use dashmap::DashMap;
use parking_lot::Mutex;

use crate::enums::Command;

pub struct Log {
    sender: Sender<Command>,
    reader: Mutex<BufReader<File>>,
    offsets: DashMap<String, usize>,
}

impl Log {
    pub fn new(path: &Path) -> Self {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(path)
            .unwrap();

        let reader = BufReader::new(file);

        let sender = spawn_writer(path);

        Self {
            sender,
            reader: Mutex::new(reader),
            offsets: DashMap::new(),
        }
    }

    pub fn append_batch(&self, messages: Vec<Bytes>) -> usize {
        let len = messages.len();
        self.sender.send(Command::Append(messages)).unwrap();

        len
    }

    pub fn read_batch(&self, consumer_id: &str, max_messages: usize) -> Vec<Bytes> {
        let mut reader = self.reader.lock();

        let mut offset = match self.offsets.get_mut(consumer_id) {
            Some(o) => o,
            None => self.offsets.entry(consumer_id.to_string()).or_insert(0),
        };

        reader.seek(SeekFrom::Start(*offset as u64)).unwrap();

        let mut messages = Vec::with_capacity(max_messages);

        for _ in 0..max_messages {
            let mut len_buf = [0u8; 4];
            if reader.read_exact(&mut len_buf).is_err() {
                break;
            }

            let len = u32::from_be_bytes(len_buf) as usize;

            let mut msg = vec![0u8; len];
            if reader.read_exact(&mut msg).is_err() {
                break;
            }

            *offset += 4 + len;
            messages.push(Bytes::from(msg));
        }

        messages
    }
}

fn spawn_writer(path: &Path) -> Sender<Command> {
    let (tx, rx) = unbounded::<Command>();
    let path = path.to_owned();

    std::thread::spawn(move || {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(path)
            .unwrap();

        let mut writer = BufWriter::with_capacity(8 << 20, file); // 8MB

        const MAX_BATCH_COMMANDS: usize = 8 * 8192;
        const FLUSH_BYTES: usize = 32 * 1024 * 1024; // 32MB
        const FLUSH_INTERVAL: Duration = Duration::from_millis(200);

        let mut buffer = Vec::with_capacity(1024);
        let mut bytes_written = 0usize;
        let mut last_flush = Instant::now();

        loop {
            match rx.recv() {
                Ok(cmd) => buffer.push(cmd),
                Err(_) => break,
            };

            while buffer.len() < MAX_BATCH_COMMANDS {
                match rx.try_recv() {
                    Ok(cmd) => buffer.push(cmd),
                    Err(_) => break,
                }
            }

            let mut batch_buf = Vec::with_capacity(1024);

            for cmd in buffer.drain(..) {
                match cmd {
                    Command::Append(messages) => {
                        for msg in messages {
                            batch_buf.extend_from_slice(&(msg.len() as u32).to_be_bytes());
                            batch_buf.extend_from_slice(&msg);
                            bytes_written += 4 + msg.len();
                        }
                    }
                }
            }

            writer.write_all(&batch_buf).unwrap();

            if bytes_written >= FLUSH_BYTES || last_flush.elapsed() >= FLUSH_INTERVAL {
                writer.flush().unwrap();
                bytes_written = 0;
                last_flush = Instant::now();
            }
        }

        writer.flush().unwrap();
    });

    tx
}
