use clap::Parser;
use std::sync::Arc;
use std::time::{Instant, Duration};

pub mod client;
pub mod utils;

use client::h3_client::ErrorStats;

#[derive(Parser)]
#[command(version, about = "HTTP/3 load testing tool")]
struct Cli {
    #[arg(long, default_value = "h3")]
    protocol: String,

    #[arg(long)]
    target: String,

    #[arg(long, default_value = "443")]
    port: u16,

    #[arg(short = 'n', long, default_value = "1000")]
    requests: usize,

    #[arg(short = 'w', long, default_value = "1", value_parser = clap::value_parser!(usize).range(1..))]
    workers: usize,
    
    #[arg(short = 't', long, default_value = "30")]
    duration: u64,
    
    #[arg(long, default_value = "/")]
    path: String,
    
    #[arg(long)]
    host: Option<String>,
    
    #[arg(long)]
    insecure: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if cli.protocol != "h3" {
        eprintln!("Only HTTP/3 supported");
        std::process::exit(1);
    }

    let host = cli.host.as_ref().unwrap_or(&cli.target).clone();

    println!("Starting HTTP/3 load test:");
    println!("  Target: {}:{}", cli.target, cli.port);
    println!("  Host: {}", host);
    println!("  Path: {}", cli.path);
    println!("  Workers: {}", cli.workers);
    println!("  Total requests: {}", cli.requests);
    println!("  Duration: {}s", cli.duration);
    println!("  Insecure: {}", cli.insecure);
    println!();

    let start_time = Instant::now();
    let deadline = start_time + Duration::from_secs(cli.duration);
    let deadline = Arc::new(deadline);
    let mut total_requests = 0;
    let mut successful_requests = 0;
    let mut failed_requests = 0;
    let mut total_errors = ErrorStats::default();

    let mut handles = vec![];

    // Distribute requests: quotient to all workers, remainder to first N workers
    let quotient = cli.requests / cli.workers;
    let remainder = cli.requests % cli.workers;

    for worker_id in 0..cli.workers {
        let target = cli.target.clone();
        let port = cli.port;
        let host = host.clone();
        let path = cli.path.clone();
        let insecure = cli.insecure;
        let requests_per_worker = quotient + if worker_id < remainder { 1 } else { 0 };
        let deadline = Arc::clone(&deadline);

        let handle = tokio::spawn(async move {
            let mut client = match client::h3_client::Http3Client::new(insecure) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Worker {}: Failed to init client: {}", worker_id, e);
                    return (0, 0, ErrorStats::default());
                }
            };

            let mut success = 0;
            let mut fail = 0;
            let mut total_errors = ErrorStats::default();

            for i in 0..requests_per_worker {
                // Check if we've exceeded the duration deadline
                if Instant::now() >= *deadline {
                    break;
                }

                match client.send_request(&target, port, &host, &path).await {
                    Ok((_body, errors)) => {
                        success += 1;
                        total_errors.send_errors += errors.send_errors;
                        total_errors.recv_errors += errors.recv_errors;
                        total_errors.quic_errors += errors.quic_errors;
                        total_errors.stream_reset_errors += errors.stream_reset_errors;
                    }
                    Err(e) => {
                        eprintln!("Worker {}: Request {} failed: {}", worker_id, i, e);
                        fail += 1;
                    }
                }
            }

            (success, fail, total_errors)
        });

        handles.push(handle);
    }

    for handle in handles {
        if let Ok((s, f, errors)) = handle.await {
            total_requests += s + f;
            successful_requests += s;
            failed_requests += f;
            total_errors.send_errors += errors.send_errors;
            total_errors.recv_errors += errors.recv_errors;
            total_errors.quic_errors += errors.quic_errors;
            total_errors.stream_reset_errors += errors.stream_reset_errors;
        }
    }

    let elapsed = start_time.elapsed().as_secs_f64();
    let hit_duration_limit = Instant::now() >= *deadline;

    println!("\nLoad test completed:");
    println!("  Total time: {:.2}s", elapsed);
    println!("  Total requests: {}", total_requests);
    println!("  Successful requests: {}", successful_requests);
    println!("  Failed requests: {}", failed_requests);
    println!("  Requests/sec: {:.2}", if elapsed > 0.0 { total_requests as f64 / elapsed } else { 0.0 });

    if hit_duration_limit {
        println!("  Completion reason: Duration limit ({:.0}s) reached", cli.duration);
    } else {
        println!("  Completion reason: All {} requests completed", cli.requests);
    }

    // Report error breakdown
    let has_errors = total_errors.send_errors > 0
        || total_errors.recv_errors > 0
        || total_errors.quic_errors > 0
        || total_errors.stream_reset_errors > 0;

    if has_errors {
        println!("\nError breakdown:");
        if total_errors.send_errors > 0 {
            println!("  Network send errors: {}", total_errors.send_errors);
        }
        if total_errors.recv_errors > 0 {
            println!("  Network recv errors: {}", total_errors.recv_errors);
        }
        if total_errors.quic_errors > 0 {
            println!("  QUIC/protocol errors: {}", total_errors.quic_errors);
        }
        if total_errors.stream_reset_errors > 0 {
            println!("  Stream reset errors: {}", total_errors.stream_reset_errors);
        }
    }

    // Verify that all requested requests were sent (only if we didn't hit duration limit)
    if !hit_duration_limit && total_requests != cli.requests {
        eprintln!(
            "Warning: Request count mismatch! Expected {}, but sent {}",
            cli.requests, total_requests
        );
        return Err(format!(
            "Request count mismatch: expected {} but sent {}",
            cli.requests, total_requests
        )
        .into());
    }

    Ok(())
}