# vex ⚡

A minimal load testing tool for HTTP/3 servers built in Rust

---

## ✨ Vision

- **HTTP/3 support** with `quiche` and direct QUIC implementation
- **Concurrent request execution** using Tokio async runtime
- **Basic metrics collection**: success/failure counts and RPS calculation
- **Self-signed certificate support** for local testing
- **Simple console output** with request statistics

---

## 🧩 Planned Features

- [x] **CLI interface** with configurable parameters
- [ ] **Rich latency metrics** (avg, p50, p95, p99, max)
- [ ] **Export results** as JSON/CSV
- [ ] **Custom headers** & **POST payloads** support
- [ ] **Connection keep-alive** vs fresh-per-request options
- [ ] **Histogram reporting** in console
- [ ] **Ramp-up traffic mode** (gradual increase)
- [ ] **Distributed load testing** across multiple nodes

---

## 🏗️ Current Implementation

The current implementation demonstrates the core architecture:

1. **HTTP/3 Client Setup**
   - Uses `quiche` library for direct QUIC/HTTP3 implementation
   - Custom `Http3Client` struct with configurable QUIC settings
   - Supports both secure and insecure connections

2. **Concurrent Worker System**
   - Spawns multiple async tasks using `tokio::spawn`
   - Each worker handles a portion of the total requests
   - Workers run for specified duration with timeout handling

3. **Basic Metrics Collection**
   - Tracks successful and failed requests
   - Calculates requests per second (RPS)
   - Measures total elapsed time

4. **Simple Reporting**
   - Console output with basic statistics
   - Success/failure counts and performance metrics

---

## 📊 Current Usage

```bash
# Compile and run with CLI parameters
RUSTFLAGS="--cfg reqwest_unstable" cargo run -- \
  --target 127.0.0.1 --port 443 --workers 10 --requests 100 --duration 30 \
  --path "/" --insecure

# Example with all options
RUSTFLAGS="--cfg reqwest_unstable" cargo run -- \
  --protocol h3 \
  --target example.com \
  --port 443 \
  --workers 50 \
  --requests 1000 \
  --duration 60 \
  --path "/api/test" \
  --host "example.com" \
  --insecure

# The `http3` feature is unstable, and requires the `RUSTFLAGS='--cfg reqwest_unstable'` environment variable to be set.
```

## 📊 CLI Usage Examples

```bash
# Run 5000 requests with 100 concurrent workers for 30 seconds
vex --workers 100 --requests 5000 --duration 30 --target proxy.spooky.local --port 9889

# Run load test for 60s with 200 workers on local service
vex --workers 200 --duration 60 --target 127.0.0.1 --port 7777 --insecure

# Test specific endpoint with custom host header
vex --workers 50 --requests 1000 --target service.local --path "/api/health" --host "service.local" --insecure

# Full example with all options
vex --protocol h3 --target example.com --port 443 --workers 100 --requests 10000 --duration 120 --path "/api/v1/users" --host "api.example.com"
```

---

## 📅 Roadmap

* **Phase 1** ✅ → HTTP/3 load testing with concurrent workers and CLI interface
* **Phase 2** → Rich latency metrics (avg, p50, p95, p99, max) and export features
* **Phase 3** → Advanced features (keep-alive, custom headers, POST support)
* **Future** → TCP/WebSocket support, distributed mode

---

## 🛠️ Dependencies

- **Tokio** - Async runtime for concurrent request handling
- **Quiche** - Rust implementation of QUIC and HTTP/3 protocols
- **Clap** - Command line argument parser
- **Rand** - Random number generation for connection IDs

---

## 📝 Development Notes

* **Rust chosen** for performance and fine-grained control over concurrency
* **HTTP/3 focus** - modern protocol with better performance characteristics
* **Quiche library** used for direct QUIC/HTTP3 protocol implementation
* **Async/await pattern** for efficient concurrent request handling
* Keep it **minimal like `wrk`** - focus on core functionality first

---

## 🔖 License

Apache 2.0