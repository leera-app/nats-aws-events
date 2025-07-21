# Lambda Orchestrator with NATS

A Rust-based event-driven system to manage and trigger AWS Lambda functions, offering features like delayed retries, observability, and future support for a user-friendly web interface to schedule and configure events similar to AWS EventBridge.

## 🚀 Overview

This project provides a serverless orchestration layer using **NATS JetStream** for managing events and **AWS Lambda** for compute. It ensures:

- Triggering Lambda functions based on events.
- Monitoring Lambda execution using **CloudWatch Logs**.
- Retrying failed executions with exponential backoff (up to 24 hours).
- Future support for UI via **Actix Web** to:
  - Provide AWS credentials.
  - Configure triggers, schedules, and rules.
  - Mimic AWS EventBridge-like routing and scheduling.

## 🛠 Components

### 1. `main.rs`
Initializes NATS connection and runs two core services in parallel:
- `lambda_trigger`: Consumes events from NATS and invokes Lambda.
- `status_checker`: Checks Lambda status and retries if failed.

### 2. `lambda_trigger.rs`
- Connects to the `my_bridge` stream in NATS.
- Subscribes to `my.event` subject.
- Invokes Lambda with the payload.
- Publishes a delayed event to `check.lambda.status` for retry tracking.

### 3. `status_checker.rs`
- Monitors delayed messages on `my.status`.
- Checks Lambda execution status from CloudWatch Logs.
- If failed, republishes the event to `my.event` with incremental delay.

## 🧪 Local Development

### Run NATS Locally (JetStream Enabled)

```bash
docker run -d --name nats-js -p 4222:4222 -p 8222:8222 -v $(pwd)/nats-data:/data nats -js -sd /data -m 8222
```

Or use Docker Compose:

```yaml
# docker-compose.yml
version: '3'
services:
  nats:
    image: nats:latest
    ports:
      - "4222:4222"
      - "8222:8222"
    volumes:
      - ./nats-data:/data
    command: -js -sd /data -m 8222
```

Then:

```bash
docker-compose up -d
```

### Run the Rust Project

```bash
cargo run
```

Ensure AWS credentials are configured via environment or AWS CLI.

## 📦 Features (Planned)

- [x] Trigger Lambda with payloads.
- [x] Retry mechanism using delayed NATS headers.
- [x] CloudWatch Logs integration.
- [ ] Actix Web UI for configuring:
  - AWS credentials
  - Rule-based Lambda routing
  - Scheduled triggers like EventBridge
- [ ] Secure credential storage
- [ ] Role-based access for managing trigger rules

## 🤝 Contributing

We welcome contributions!

### Steps:
1. Fork the repo
2. Create your feature branch (`git checkout -b feature/YourFeature`)
3. Commit your changes (`git commit -am 'Add new feature'`)
4. Push to the branch (`git push origin feature/YourFeature`)
5. Open a Pull Request

### Areas to Contribute:
- UI using Actix Web
- Enhanced error handling
- Support for other AWS services
- Unit & integration tests

## 📄 License

This project is licensed under the MIT License.

---

Built with ❤️ in Rust using NATS and AWS Lambda.
