use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use reqwest::Client;
use reqwest::StatusCode;
use tokio::time::{Duration, Instant};

const BASE_URL: &str = "http://localhost:3000";
const NUM_TOPICS: usize = 10;
/// Distinct `consumer_id` values per topic (independent offsets = separate consumer groups).
const NUM_CONSUMER_GROUPS: usize = 5;

fn encode_messages(batch_size: usize) -> Vec<u8> {
    let mut buf = Vec::with_capacity(batch_size * (4 + 5));

    for _ in 0..batch_size {
        let msg = b"hello";
        let len = msg.len() as u32;

        buf.extend_from_slice(&len.to_be_bytes());
        buf.extend_from_slice(msg);
    }

    buf
}

fn count_messages(body: &[u8]) -> usize {
    let mut offset = 0;
    let mut count = 0;

    while offset + 4 <= body.len() {
        let len = u32::from_be_bytes(body[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;

        if offset + len > body.len() {
            break;
        }

        offset += len;
        count += 1;
    }

    count
}

#[tokio::main]
async fn main() {
    let client = Client::new();

    let producer_workers = 100;
    let messages_per_worker = 10_000;
    let batch_size = 100;

    let producers_done = Arc::new(AtomicBool::new(false));
    let total_pulled = Arc::new(AtomicU64::new(0));

    let mut consumer_handles = Vec::new();

    for topic_idx in 0..NUM_TOPICS {
        for group_idx in 0..NUM_CONSUMER_GROUPS {
            let client = client.clone();
            let producers_done = producers_done.clone();
            let total_pulled = total_pulled.clone();
            let base = BASE_URL.to_string();

            consumer_handles.push(tokio::spawn(async move {
                let topic = format!("topic-{}", topic_idx);
                let consumer_id = format!("group-{}", group_idx);
                let url = format!(
                    "{}/pull?topic={}&consumer_id={}&batch={}",
                    base, topic, consumer_id, batch_size
                );

                loop {
                    let Ok(resp) = client.get(&url).send().await else {
                        tokio::time::sleep(Duration::from_millis(5)).await;
                        continue;
                    };

                    let n = if resp.status() == StatusCode::NOT_FOUND {
                        0
                    } else {
                        let Ok(bytes) = resp.bytes().await else {
                            continue;
                        };
                        count_messages(&bytes)
                    };

                    total_pulled.fetch_add(n as u64, Ordering::Relaxed);

                    if producers_done.load(Ordering::Acquire) && n == 0 {
                        break;
                    }

                    if n == 0 {
                        tokio::time::sleep(Duration::from_millis(1)).await;
                    }
                }
            }));
        }
    }

    let start = Instant::now();
    let mut producer_handles = Vec::new();

    for i in 0..producer_workers {
        let client = client.clone();
        let topic_id = i % NUM_TOPICS;

        producer_handles.push(tokio::spawn(async move {
            for _ in 0..(messages_per_worker / batch_size) {
                let body = encode_messages(batch_size);
                let url = format!(
                    "{}/push?topic=topic-{}",
                    BASE_URL, topic_id
                );
                let _ = client.post(&url).body(body).send().await;
            }
        }));
    }

    for h in producer_handles {
        h.await.unwrap();
    }

    let producer_phase = start.elapsed();
    producers_done.store(true, Ordering::Release);

    for h in consumer_handles {
        h.await.unwrap();
    }

    let total_elapsed = start.elapsed();
    let total_messages = producer_workers * messages_per_worker;
    let pulled = total_pulled.load(Ordering::Relaxed);

    println!("Topics: {}", NUM_TOPICS);
    println!(
        "Consumer groups per topic (distinct consumer_id): {}",
        NUM_CONSUMER_GROUPS
    );
    println!("Pull workers: {}", NUM_TOPICS * NUM_CONSUMER_GROUPS);
    println!("Sent: {}", total_messages);
    println!(
        "Pulled (sum across groups; each group reads the full topic): {}",
        pulled
    );
    println!("Producer phase: {:.2}s", producer_phase.as_secs_f64());
    println!("Total (incl. drain): {:.2}s", total_elapsed.as_secs_f64());
    println!(
        "Push throughput: {:.0} msg/sec",
        total_messages as f64 / producer_phase.as_secs_f64()
    );
    println!(
        "Overall pull rate: {:.0} msg/sec",
        pulled as f64 / total_elapsed.as_secs_f64()
    );
}
