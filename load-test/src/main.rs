use clap::Parser;
use reqwest::Client;
use tokio::time::Instant;

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    topics: bool,
}

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

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let client = Client::new();

    let workers = 100;
    let messages_per_worker = 10_000;
    let batch_size = 100;

    let start = Instant::now();
    let mut handles = Vec::new();

    for i in 0..workers {
        let client = client.clone();
        let topics_enabled = args.topics;

        let handle = tokio::spawn(async move {
            for _j in 0..(messages_per_worker / batch_size) {
                let body = encode_messages(batch_size);

                // choose topic (if enabled)
                let url = if topics_enabled {
                    let topic_id = i % 10; // 10 topics
                    format!("http://localhost:3000/push?topic=topic-{}", topic_id)
                } else {
                    "http://localhost:3000/push".to_string()
                };

                let _ = client.post(&url).body(body.clone()).send().await;
            }
        });

        handles.push(handle);
    }

    for h in handles {
        h.await.unwrap();
    }

    let duration = start.elapsed().as_secs_f64();
    let total_messages = workers * messages_per_worker;

    println!(
        "Mode: {}",
        if args.topics {
            "topics"
        } else {
            "single queue"
        }
    );
    println!("Sent: {}", total_messages);
    println!("Time: {:.2}s", duration);
    println!(
        "Throughput: {:.0} msg/sec",
        total_messages as f64 / duration
    );
}
