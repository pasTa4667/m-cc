use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::Mutex;

use bytes::Bytes;
use dashmap::DashMap;

pub struct Log {
    writer: Mutex<BufWriter<File>>,
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

        let writer = BufWriter::new(file.try_clone().unwrap());
        let reader = BufReader::new(file.try_clone().unwrap());

        Self {
            writer: Mutex::new(writer),
            reader: Mutex::new(reader),
            offsets: DashMap::new(),
        }
    }

    pub fn append_batch(&self, messages: &Vec<Bytes>) -> usize {
        let mut writer = self.writer.lock().unwrap();

        for msg in messages {
            let len = msg.len() as u32;
            writer.write_all(&len.to_be_bytes()).unwrap();
            writer.write_all(msg).unwrap();
        }

        messages.len()
        // writer.flush().unwrap();
    }

    pub fn read_batch(&self, consumer_id: &str, max_messages: usize) -> Vec<Bytes> {
        let mut reader = self.reader.lock().unwrap();

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
