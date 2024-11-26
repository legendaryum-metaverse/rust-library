<div align="center">
  <h1>Legend saga</h1>
  <p>
    <strong>A Rust library that simplifies microservice communication via RabbitMQ, supporting event-driven patterns and reliable messaging.</strong>
  </p>
  <p>

<!-- prettier-ignore-start -->

[![crates.io](https://img.shields.io/crates/v/legend-saga?label=latest)](https://crates.io/crates/legend-saga)
[![Documentation](https://docs.rs/legend-saga/badge.svg?version=latest)](https://docs.rs/legend-saga)
[![dependency status](https://deps.rs/repo/github/legendaryum-metaverse/rust-library/status.svg)](https://deps.rs/repo/github/legendaryum-metaverse/rust-library)
<br />
[![CI](https://github.com/legendaryum-metaverse/rust-library/actions/workflows/main.yml/badge.svg)](https://github.com/legendaryum-metaverse/rust-library/actions/workflows/main.yml)
![downloads](https://img.shields.io/crates/d/legend-saga.svg)

<!-- prettier-ignore-end -->

  </p>
</div>

---

## Features

**Core Communication:**

- **Publish/Subscribe Messaging:** Exchange messages between microservices using a
  publish-subscribe pattern.
- **Headers-Based Routing:** Leverage the power of RabbitMQ's headers exchange for flexible and dynamic routing of messages based on custom headers.
- **Durable Exchanges and Queues:** Ensure message persistence and reliability with durable RabbitMQ components.

**Saga Management:**

<div style="text-align: center;">
<img src="https://raw.githubusercontent.com/legendaryum-metaverse/legend-transactional/main/.github/assets/saga.png" alt="legendaryum" style="width: 90%;"/>
</div>

- **Saga Orchestration:** Coordinate complex, multi-step transactions across multiple microservices with saga orchestration.
- **Saga Step Handlers:** Implement step-by-step saga logic in your microservices using callbacks.
- **Compensation Logic:** Define compensating actions for saga steps to handle failures
  gracefully and maintain data consistency.

## Flags

`std` y `events`features flags:

- `std` is the main app,
- `events = ["serde", "strum", "strum_macros"]` are used to handle types, payloads, enum, struct of the app.

## Contributors

Thanks to [all contributors](https://github.com/legendaryum-metaverse/rust-library/graphs/contributors)!

## Author

Jorge Clavijo <https://github.com/jym272>

## License

Distributed under the MIT License. See [LICENSE](LICENSE) for more information.
