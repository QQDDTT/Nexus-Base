use std::time::{Instant, SystemTime, UNIX_EPOCH};
use std::sync::Arc;
use tokio::task;
use tonic::Request;
use ed25519_dalek::{Signer, SigningKey};
use rand::Rng;

use nexus_proto::nexus::v1::nexus_knowledge_service_client::NexusKnowledgeServiceClient;
use nexus_proto::nexus::v1::{RetrieveRequest, IngestRequest, MemoryFragment, CommitRequest};

const CONCURRENCY: usize = 50;
const TOTAL_RETRIEVE: usize = 1000;
const PRE_INGEST_COUNT: usize = 5000; // 为了速度，默认生成 5000 条

fn generate_random_tensor(dim: usize) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect()
}

fn sign_request<T>(req: &mut Request<T>, key_path: &str) {
    let hex_key = std::fs::read_to_string(key_path).expect("Failed to read private key. Please run 'nexus-cli keygen' first.");
    let bytes = hex::decode(hex_key.trim()).unwrap();
    let signing_key = SigningKey::from_bytes(bytes.as_slice().try_into().unwrap());
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs().to_string();
    let signature = signing_key.sign(timestamp.as_bytes());
    let sig_hex = hex::encode(signature.to_bytes());
    
    req.metadata_mut().insert("authorization", sig_hex.parse().unwrap());
    req.metadata_mut().insert("x-timestamp", timestamp.parse().unwrap());
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=============================================");
    println!("   Nexus-Base Benchmark (Mmap & Ingest)      ");
    println!("=============================================");

    let endpoint = "http://[::1]:9090";
    let mut client = NexusKnowledgeServiceClient::connect(endpoint).await.expect("Server is not running!");

    let key_path = "./private.key"; // Run from workspace root

    // 1. 预热灌入数据
    println!("\n>> Phase 1: Pre-ingesting {} records to build Mmap snapshot...", PRE_INGEST_COUNT);
    let start_ingest = Instant::now();
    for i in 0..PRE_INGEST_COUNT {
        let mut req = Request::new(IngestRequest {
            fragments: vec![MemoryFragment {
                id: format!("bench-doc-{}", i),
                file_path: format!("/fake/path/{}", i),
                content_snippet: format!("This is a benchmarking string number {}", i),
                embedding: generate_random_tensor(768),
                metadata_anchors: vec!["benchmark".to_string()],
                similarity_score: 0.0,
            }],
        });
        sign_request(&mut req, key_path);
        client.ingest_experience(req).await?;
        
        if (i + 1) % 1000 == 0 {
            println!("   Ingested {} records...", i + 1);
        }
    }
    let ingest_duration = start_ingest.elapsed();
    println!("   => Ingest QPS: {:.2} req/sec", PRE_INGEST_COUNT as f64 / ingest_duration.as_secs_f64());

    // 强制 Commit 生成快照
    println!(">> Forcing Snapshot Commit...");
    let mut commit_req = Request::new(CommitRequest { admin_signature: "".to_string() });
    sign_request(&mut commit_req, key_path);
    client.commit_snapshot(commit_req).await?;
    println!("   => Snapshot written and loaded via Mmap.");

    // 2. 并发检索压测 (Mmap Retrieve Benchmark)
    println!("\n>> Phase 2: Mmap Retrieve Benchmark");
    println!("   Concurrency: {}, Total Requests: {}", CONCURRENCY, TOTAL_RETRIEVE);
    
    let mut tasks = Vec::new();
    let requests_per_task = TOTAL_RETRIEVE / CONCURRENCY;
    let endpoint_str = endpoint.to_string();

    let start_bench = Instant::now();

    for _ in 0..CONCURRENCY {
        let ep = endpoint_str.clone();
        tasks.push(task::spawn(async move {
            let mut c = NexusKnowledgeServiceClient::connect(ep).await.unwrap();
            let mut latencies = Vec::with_capacity(requests_per_task);
            
            for _ in 0..requests_per_task {
                let req_start = Instant::now();
                let query = generate_random_tensor(768);
                let req = Request::new(RetrieveRequest {
                    query_tensor: query,
                    anchors: vec![],
                    top_k: 10,
                });
                
                c.retrieve_context(req).await.unwrap();
                latencies.push(req_start.elapsed().as_millis());
            }
            latencies
        }));
    }

    let mut all_latencies = Vec::new();
    for t in tasks {
        let res = t.await?;
        all_latencies.extend(res);
    }
    
    let bench_duration = start_bench.elapsed();
    all_latencies.sort();
    
    let total_time_secs = bench_duration.as_secs_f64();
    let qps = TOTAL_RETRIEVE as f64 / total_time_secs;
    let avg_latency = all_latencies.iter().sum::<u128>() as f64 / all_latencies.len() as f64;
    let p95_latency = all_latencies[(all_latencies.len() as f64 * 0.95) as usize];
    let p99_latency = all_latencies[(all_latencies.len() as f64 * 0.99) as usize];

    println!("\n=============================================");
    println!("Benchmark Results:");
    println!("Time Taken: {:.2} seconds", total_time_secs);
    println!("Retrieve QPS: {:.2} req/sec", qps);
    println!("Average Latency: {:.2} ms", avg_latency);
    println!("P95 Latency: {} ms", p95_latency);
    println!("P99 Latency: {} ms", p99_latency);
    println!("=============================================");

    Ok(())
}
