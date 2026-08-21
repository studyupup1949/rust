# actix_tower Enterprise Roadmap

The `actix_tower` crate has successfully bridged the notorious concurrency gap between Actix Web (`!Send` workers) and the Tower ecosystem (`Send` middleware). 

As we scale toward the `v1.0.0` stabilization and beyond, our roadmap is strictly focused on **Enterprise Adoption**, **Maximum Backwards Compatibility**, and **Big Tech Integrations** (AWS, Cloudflare, Meta).

## Core Philosophy: Maximum Community Adoption
`actix_tower` will **always** prioritize community reach. We will not force users onto bleeding-edge compilers if it breaks enterprise compliance.
- **Stable Rust Support:** The core architecture will continuously support older Rust compilers (2021/2022).
- **Opt-In Modernity:** Any new Rust 2024 features will be placed behind explicit `Cargo.toml` feature flags to guarantee 100% backwards compatibility for legacy enterprise codebases.

---

## 1. Targeting "Big Tech" Infrastructure

To be adopted by major tech companies, the crate must seamlessly integrate with their strict operational requirements.

### Native OpenTelemetry Context Carrier Propagation
Observability is a strict requirement for distributed microservices. When a request jumps the boundary from Actix Web's single-threaded HTTP engine into the `RefCell` pool to execute Tower middleware, tracing spans are highly susceptible to losing their execution context (`tracing::Span::current()`).
- **The Goal:** Build a built-in tracing middleware layer that intercepts the tracing context, packages the OpenTelemetry Context into an explicit pointer container, and forces propagation across the `Send` / `!Send` boundary.
- **The Value:** This makes `actix_tower` a drop-in production tool for infrastructure teams operating vast mixed-framework setups (like Tonic gRPC + Actix Web HTTP) on AWS X-Ray or DataDog.

### Hard Performance Benchmarks (Data Over Hype)
Enterprise architects obsess over milliseconds and throughput.
- **The Goal:** Build an exhaustive `criterion` benchmark suite that explicitly proves `actix_tower` can handle 100,000+ requests per second with negligible latency overhead compared to raw Actix.
- **The Value:** Quantifiable proof that the `RefCell` worker pool does not degrade Actix Web's legendary single-node performance.

### Cloud-Native Examples (AWS / Azure)
- **The Goal:** Create enterprise-grade integration examples (e.g., `examples/aws_lambda_microservice.rs` or integrations with the official AWS Rust SDK).
- **The Value:** Directly captures search traffic from AWS engineers looking for robust Actix + Tower implementation patterns.

---

## 2. Advanced Architectural Features

### Adaptive Backpressure and Load-Shedding Co-ordination
A major danger when bridging boundaries is queue starvation or pool exhaustion under massive traffic spikes (DDoS or viral load).
- **The Goal:** Deeply integrate with Tower’s load-shedding and structural rate-limiting crates (`tower::limit`, `tower::load_shed`), tightly coordinating them with Actix's internal worker status.
- **The Value:** If the worker pool reaches peak capacity, it will signal `Poll::Pending` back up to Actix's listener layer, gracefully dropping requests at the edge before the `RefCell` pool deadlocks under memory exhaustion.

### Feature-Gated Zero-Cost `AsyncFn` Extraction (Rust 2024)
The Rust web ecosystem is transitioning rapidly toward the new `AsyncFn` trait family in the 2024 Edition.
- **The Goal:** Implement a zero-allocation adapter macro that bridges Actix's powerful extractor system directly into `tower::Service` requests using static dispatch (`impl AsyncFn`), bypassing `BoxFuture` heap allocations.
- **The Strategy:** This will be hidden behind a `rust_2024_async` Cargo feature flag. It provides zero-cost abstractions for developers on the newest compilers without breaking compatibility for enterprises on older compilers.
