# Legend-Saga Library Development Guide

> Production-ready Rust library for microservice communication via RabbitMQ with saga patterns and audit logging

## What is This?

**legend-saga** (v0.0.41) is a Rust library providing event-driven microservice communication through RabbitMQ. It implements:

- **Event-driven messaging** with headers-based routing
- **Saga patterns** for distributed transactions (orchestration + compensation)
- **Audit logging** tracking full event lifecycle (received, processed, dead-letter)
- **Resilient messaging** with fibonacci backoff and retry strategies
- **Production-grade testing** with containerized infrastructure

## Quick Start

```bash
make format    # Format code
make lint-fix  # Fix linting issues
make test      # Run full test suite (starts Docker, runs tests)
make all       # Run everything (prettier + format + lint-fix + test)
```

**CRITICAL**: Tests require `--test-threads=1` due to RabbitMQ containers. The `make test` command (via `scripts/test.sh`) handles this automatically:
```bash
# scripts/test.sh does:
docker compose up -d
export LOG_LEVEL=info
cargo test --lib -- --test-threads=1 --nocapture
```

## Architecture Overview

### Core Concepts

1. **Event System**: Type-safe events defined in `MicroserviceEvent` enum
2. **Audit Trail**: Automatic tracking via `AuditReceived`, `AuditProcessed`, `AuditDeadLetter` events
3. **Exchange Patterns**:
   - Topic exchanges for fan-out (pub/sub)
   - Direct exchanges for single-consumer (audit events)
4. **Handler Pattern**: `EventHandler` trait for processing + `AuditHandler` (prevents recursive audits)
5. **Connection Management**: Per-microservice connections with shared state (Arc/Mutex)

### Event Flow

```
Event Published → Event Received → Audit.Received emitted
                       ↓
                 Event Processing
                       ↓
        ┌──────────────┴──────────────┐
        ↓                             ↓
   Success: Audit.Processed      Failure: Audit.DeadLetter
        ↓                             ↓
     ACK event                    NACK event
```

**Key**: Audit events use direct exchanges to avoid recursive audit loops. The `AuditHandler` never emits audit events.

## Critical Development Rules

### Testing Requirements

⚠️ **ALWAYS use `make test` for running tests**. Direct `cargo test` will fail without proper setup.

For debugging individual tests:
```bash
LOG_LEVEL=info cargo test --lib test_name -- --test-threads=1 --nocapture --color always
```

Why `--test-threads=1`? RabbitMQ containers are shared across tests and not thread-safe.

### Adding New Features

**New Events**:
1. Add variant to `MicroserviceEvent` enum (`events.rs`)
2. Create payload struct implementing `Serialize`/`Deserialize`
3. Update routing keys in `queue_consumer_props.rs`
4. Implement `EventHandler` for processing logic

**New Microservices**:
1. Add to `AvailableMicroservices` enum
2. Define queue properties in `queue_consumer_props.rs`
3. Use `RabbitMQClient::microservice_consumer()` in `start.rs`

**Audit Events**:
- Automatically emitted for all non-audit events
- Use `AuditHandler` for consuming audit events (prevents recursion)
- Audit payloads include: microservice name, event type, timestamp

### Code Patterns

**DON'T**:
- Emit audit events from `AuditHandler` (causes infinite loops)
- Hard-code queue/exchange names (use `QueueConsumerProps`)
- Skip error handling in event handlers
- Run tests without `--test-threads=1`

**DO**:
- Use `TestSetup` for integration tests
- Leverage `RabbitMQError` for error propagation
- Follow async patterns with Arc/Mutex for shared state
- Check existing patterns before implementing new features

## Key Files Reference

| File                     | Purpose                          | Common Tasks                       |
|--------------------------|----------------------------------|------------------------------------|
| `events.rs`              | Event type definitions           | Add new events, payloads           |
| `events_consume.rs`      | Core event handling logic        | Modify processing, ack/nack        |
| `start.rs`               | Connection management            | Add microservice consumers         |
| `consumers.rs`           | RabbitMQ infrastructure          | Exchange/queue setup               |
| `publish_event.rs`       | Event publishing methods         | Add new publish patterns           |
| `nack.rs`                | Retry/dead-letter strategies     | Adjust backoff algorithms          |
| `queue_consumer_props.rs`| Queue/routing configuration      | Define new queues                  |
| `test.rs`                | Test utilities and setup         | Enhance test infrastructure        |
| `connection.rs`          | Low-level RabbitMQ connections   | Connection pooling, channels       |

## Common Tasks

### Running Tests
```bash
make test                    # Full suite with Docker setup
make test-compile           # Check tests compile without running
```

### Debugging
```bash
# Enable detailed logging
LOG_LEVEL=debug cargo test --lib test_name -- --test-threads=1 --nocapture

# Check RabbitMQ containers
docker compose ps
docker compose logs rabbitmq
```

### Adding a New Event Type
```rust
// 1. events.rs - Add enum variant
pub enum MicroserviceEvent {
    // ...
    MyNewEvent,
}

// 2. events.rs - Create payload
#[derive(Serialize, Deserialize)]
pub struct MyNewEventPayload {
    pub field: String,
}

// 3. queue_consumer_props.rs - Add routing
// Update relevant QueueConsumerProps

// 4. Implement handler
pub struct MyEventHandler;
impl EventHandler for MyEventHandler {
    async fn handle(&self, delivery: Delivery) -> Result<(), RabbitMQError> {
        // Process event
    }
}

// 5. Write tests in start.rs or separate test module
```

### Checking What Changed
```bash
git diff main                # See all changes vs main branch
git log --oneline -10       # Recent commits
```

## Troubleshooting

**Tests hang/fail**: Ensure RabbitMQ containers are running (`docker compose up -d`)
**Connection errors**: Check `docker compose logs rabbitmq` for RabbitMQ issues
**Audit loops**: Verify `AuditHandler` is used for audit event consumption
**Flaky tests**: Confirm `--test-threads=1` is set

---

**Pro tip**: The codebase is well-tested and follows consistent patterns. When implementing new features, find similar existing code and follow the same structure.
