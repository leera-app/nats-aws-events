# NATS AWS Events 🚀

NATS AWS Events is a lightweight, event-driven execution system built on **NATS JetStream** and **AWS Lambda**. It allows you to reliably trigger Lambda functions with built-in **delayed retries**, **status checking**, and **streaming-based orchestration** — without relying on SQS or EventBridge.

---

## 🌟 Features

- ✅ Trigger AWS Lambda functions from NATS messages
- 🔁 Automatic retry logic with increasing delays
- ⏱️ Delayed message scheduling using JetStream headers
- 📡 Lightweight and cost-effective: no SQS, EventBridge, or DBs
- 🔧 Built in Rust with async support and AWS SDK

---

## 🛠️ Architecture Overview

1. `run_lambda_trigger` listens to the `my.event` subject and invokes AWS Lambda based on incoming payloads.
2. It schedules a status-check event to `check.lambda.status` with a delayed message.
3. `run_status_checker` checks the status of the invoked Lambda (stubbed in example) and re-sends the event to `my.event` if it failed.
4. Retries are capped and spaced out intelligently using JetStream headers.

---

## 🚀 Getting Started

### Prerequisites

- Rust (stable)
- Docker (for local NATS setup)
- AWS credentials with Lambda invoke permissions
- NATS JetStream running locally or on a server

### Clone and Run

```bash
git clone https://github.com/leera-app/nats-aws-events.git
cd nats-aws-events
cargo build
./run_all.sh
```
### Run NATS locally with JetStream
```bash
docker compose up -d
```


## 🧪 Development

### Project is organized into:

- lambda_trigger.rs: listens to events and invokes Lambda
- status_checker.rs: checks status and retries failed invocations


## 🤝 Contributing

We welcome contributions from the community! Here’s how you can help:

Fork the repo
Create a feature branch: git checkout -b my-feature
Commit your changes: git commit -m 'Add cool feature'
Push and open a Pull Request
Please make sure your code is well-tested and documented.


## 📄 License

This project is licensed under the MIT License.


## 🙏 Acknowledgements

- NATS.io
- AWS Lambda
- Rust Language