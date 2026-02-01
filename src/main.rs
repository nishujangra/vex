
use clap::Parser;
use std::time::{Duration, Instant};

pub mod client;

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

    #[arg(short = 'w', long, default_value = "1")]
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
        eprintln!("Currently only HTTP/3 (h3) protocol is supported");
        std::process::exit(1);
    }

    let connect_addr = format!("{}:{}", cli.target, cli.port);
    let host = cli.host.as_ref().unwrap_or(&cli.target).clone();
    let path = cli.path.clone();

    println!("Starting HTTP/3 load test:");
    println!("  Target: {}", connect_addr);
    println!("  Host: {}", host);
    println!("  Path: {}", path);
    println!("  Workers: {}", cli.workers);
    println!("  Total requests: {}", cli.requests);
    println!("  Duration: {}s", cli.duration);
    println!("  Insecure: {}", cli.insecure);
    println!();

    let start_time = Instant::now();
    let mut total_requests = 0;
    let mut successful_requests = 0;
    let mut failed_requests = 0;

    // Spawn worker tasks
    let mut handles = vec![];

    for worker_id in 0..cli.workers {
        let connect_addr = connect_addr.clone();
        let host = host.clone();
        let path = path.clone();
        let insecure = cli.insecure;

        let handle = tokio::spawn(async move {
            let mut client = match client::h3_client::Http3Client::new(insecure) {
                Ok(client) => client,
                Err(e) => {
                    eprintln!("Worker {}: Failed to create HTTP/3 client: {}", worker_id, e);
                    return (0, 0);
                }
            };

            let mut worker_successful = 0;
            let mut worker_failed = 0;
            let requests_per_worker = cli.requests / cli.workers;
            let start_time = Instant::now();

            for i in 0..requests_per_worker {
                if start_time.elapsed() >= Duration::from_secs(cli.duration) {
                    break;
                }

                match client.send_request(&connect_addr, &host, &path).await {
                    Ok(_response) => {
                        worker_successful += 1;
                    }
                    Err(e) => {
                        eprintln!("Worker {}: Request {} failed: {}", worker_id, i, e);
                        worker_failed += 1;
                    }
                }
            }

            (worker_successful, worker_failed)
        });

        handles.push(handle);
    }

    // Wait for all workers to complete
    for handle in handles {
        match handle.await {
            Ok((successful, failed)) => {
                successful_requests += successful;
                failed_requests += failed;
                total_requests += successful + failed;
            }
            Err(e) => {
                eprintln!("Worker task failed: {}", e);
            }
        }
    }

    let elapsed = start_time.elapsed();
    let total_seconds = elapsed.as_secs_f64();

    println!();
    println!("Load test completed:");
    println!("  Total time: {:.2}s", total_seconds);
    println!("  Total requests: {}", total_requests);
    println!("  Successful requests: {}", successful_requests);
    println!("  Failed requests: {}", failed_requests);
    println!("  Requests per second: {:.2}",
             if total_seconds > 0.0 { total_requests as f64 / total_seconds } else { 0.0 });

    Ok(())
}