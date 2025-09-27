#[cfg(test)]
pub(crate) mod setup {

    fn get_log_level() -> Level {
        let key = "LOG_LEVEL";
        if let Ok(value) = env::var(key) {
            if !value.is_empty() {
                if let Ok(level) = value.trim().to_uppercase().parse() {
                    return level;
                }
            }
        }
        Level::INFO
    }
    pub fn tracing_subscriber() {
        tracing_subscriber::fmt()
            .with_max_level(get_log_level())
            .init();
    }
    #[ctor::ctor]
    fn init() {
        if let Ok(value) = env::var("LOG_LEVEL") {
            if !value.is_empty() {
                tracing_subscriber()
            }
        }
    }

    use crate::events::MicroserviceEvent;
    use futures::Stream;
    use futures::StreamExt;
    use lapin::options::{
        BasicAckOptions, BasicConsumeOptions, BasicPublishOptions, QueueDeclareOptions,
        QueueDeleteOptions,
    };

    use crate::connection::{AvailableMicroservices, RabbitMQClient, RabbitMQError};
    use lapin::topology::TopologyDefinition;
    use lapin::types::FieldTable;
    use lapin::BasicProperties;
    use rand::distr::StandardUniform;
    use rand::prelude::Distribution;
    use rand::Rng;
    use serde::de::DeserializeOwned;
    use serde::Serialize;
    use std::env;
    use tokio::runtime::Runtime;
    use tracing::{debug, error, info, Level};

    pub const TEST_QUEUE: &str = "test_queue";
    pub const RABBIT_URI: &str = "amqp://rabbit:1234@localhost:5672";

    pub struct TestSetup {
        pub rt: Runtime,
        pub client: RabbitMQClient,
    }

    impl RabbitMQClient {
        #[cfg(test)]
        pub(crate) async fn declare_queue(
            &self,
            queue_name: &str,
            options: QueueDeclareOptions,
            arguments: FieldTable,
        ) -> Result<(), RabbitMQError> {
            let channel = self.events_channel.lock().await;
            channel
                .queue_declare(queue_name, options, arguments)
                .await?;
            info!("Queue declared: {}", queue_name);
            Ok(())
        }

        #[cfg(test)]
        pub(crate) async fn publish_message<T: Serialize>(
            &self,
            queue_name: &str,
            payload: &T,
            properties: BasicProperties,
        ) -> Result<(), RabbitMQError> {
            let serialized = serde_json::to_vec(payload)?;
            let channel = self.events_channel.lock().await;
            channel
                .basic_publish(
                    "",
                    queue_name,
                    BasicPublishOptions::default(),
                    &serialized,
                    properties,
                )
                .await?;
            info!("Message published to queue: {}", queue_name);
            Ok(())
        }

        #[cfg(test)]
        #[allow(dead_code)]
        pub(crate) async fn delete_queue(&self, queue_name: &str) -> Result<(), RabbitMQError> {
            let channel = self.events_channel.lock().await;
            channel
                .queue_delete(queue_name, QueueDeleteOptions::default())
                .await?;
            info!("Queue deleted: {}", queue_name);
            Ok(())
        }
        #[cfg(test)]
        pub(crate) async fn consume_messages<T: DeserializeOwned>(
            &self,
            queue_name: &str,
            options: BasicConsumeOptions,
        ) -> Result<impl Stream<Item = Result<T, RabbitMQError>>, RabbitMQError> {
            let channel = self.events_channel.lock().await;
            let consumer = channel
                .basic_consume(queue_name, "my_consumer", options, FieldTable::default())
                .await?;

            info!("Started consuming messages from queue: {}", queue_name);

            Ok(consumer.map(move |delivery| match delivery {
                Ok(delivery) => match serde_json::from_slice(&delivery.data) {
                    Ok(parsed) => {
                        tokio::spawn(async move {
                            if let Err(e) = delivery.ack(BasicAckOptions::default()).await {
                                error!("Failed to acknowledge message: {:?}", e);
                            }
                        });
                        Ok(parsed)
                    }
                    Err(e) => {
                        error!("Failed to deserialize message: {:?}", e);
                        Err(RabbitMQError::SerializationError(e))
                    }
                },
                Err(err) => {
                    error!("Error receiving message: {:?}", err);
                    Err(RabbitMQError::from(err))
                }
            }))
        }
    }

    pub struct Config {
        pub events: &'static [MicroserviceEvent],
        pub microservice: AvailableMicroservices,
    }

    impl Distribution<AvailableMicroservices> for StandardUniform {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> AvailableMicroservices {
            use strum::IntoEnumIterator;
            AvailableMicroservices::iter()
                .nth(rng.random_range(0..AvailableMicroservices::iter().count()))
                .unwrap()
        }
    }

    pub fn random_microservice() -> AvailableMicroservices {
        let mut rng = rand::rng();
        rng.random()
    }

    impl TestSetup {
        pub fn new(conf: Option<Config>) -> Self {
            let conf = conf.unwrap_or_else(|| Config {
                events: &[],
                microservice: random_microservice(),
            });
            let rt = Runtime::new().unwrap();
            let client = rt.block_on(async {
                RabbitMQClient::new(RABBIT_URI, conf.microservice, Some(conf.events))
                    .await
                    .expect("Failed to create RabbitMQ client")
            });
            TestSetup { rt, client }
        }

        pub(crate) async fn get_current_topology(&self) -> TopologyDefinition {
            self.client
                .current_connection()
                .await
                .expect("Cannot get the connection")
                .read()
                .await
                .topology()
        }
        /// Asynchronously cleans up the RabbitMQ topology.
        ///
        /// This function is used to clean the RabbitMQ topology (queues and exchanges) by deleting all
        /// the declared entities in the provided or current topology. If no topology is provided,
        /// the function will retrieve and use the current topology from the active connection.
        ///
        /// # Parameters
        /// - `t`: An `Option<TopologyDefinition>` representing the topology to clean.
        ///        If `None`, the current connection's topology is used.
        ///
        /// # Functionality
        /// 1. Retrieves the current connection from the client. If the connection is not established,
        ///    it produces an error as it is assumed to always exist (`unreachable!`).
        /// 2. Creates a channel using the current connection to perform cleanup operations.
        /// 3. Deletes all queues defined in the topology:
        ///    - Logs the deletion for each queue.
        ///    - Deletes the queue using `queue_delete()`.
        /// 4. Deletes all exchanges defined in the topology:
        ///    - Logs the deletion for each exchange.
        ///    - Deletes the exchange using `exchange_delete()`.
        /// 5. Ensures the cleanup channel (`delete_channel`) is properly closed after processing, while
        ///    logging any errors that occur during the channel closure process.
        ///
        /// # Logging
        /// - Logs the name of each queue and exchange being deleted.
        /// - Logs any errors encountered while closing the `delete_channel`.
        ///
        /// # Errors
        /// - Panics if the connection cannot be retrieved or if any unexpected failure occurs during
        ///   channel creation or deleting entities.
        /// - Any failure in closing the `delete_channel` is logged but does not cause a panic.
        ///
        /// # Example Usage
        /// ```rust
        /// // Assuming `client` is an instance of RabbitMQClient
        /// let topology_definition = Some(some_topology_definition);
        /// client.clean_topology(topology_definition).await;
        /// ```
        ///
        /// # Notes & Assumptions
        /// - It is assumed that the active connection is available when this function is called, as
        ///   guaranteed by `RabbitMQClient::get_connection`.
        /// - The `topology` parameter is optional; if not provided, the current connection's topology will
        ///   be used.
        pub(crate) async fn clean_topology(&self, t: Option<TopologyDefinition>) {
            let conn = self
                .client
                .current_connection()
                .await
                .expect("Cannot get the connection")
                .read()
                .await;
            if !conn.status().connected() {
                unreachable!("Connection is always guaranteed in `RabbitMQClient::get_connection`");
            }
            let delete_channel = conn.create_channel().await.unwrap();
            let t = t.unwrap_or_else(|| conn.topology());
            for queue in t.queues {
                debug!("DELETING QUEUE: {}", queue.name.to_string());
                delete_channel
                    .queue_delete(&queue.name.to_string(), QueueDeleteOptions::default())
                    .await
                    .unwrap();
            }

            // Delete all Exchanges
            for exchange in t.exchanges {
                debug!("DELETING EXCHANGE: {}", exchange.name.to_string());
                delete_channel
                    .exchange_delete(&exchange.name.to_string(), Default::default())
                    .await
                    .unwrap();
            }
            // Properly close the delete_channel before returning
            if let Err(e) = delete_channel.close(0, "Topology cleanup complete").await {
                debug!("Error closing delete_channel: {:?}", e);
                // Don't panic here as it might already be closing
            }
            debug!("RESTORED TOPOLOGY");
        }
    }

    // Rust's RAII (Resource Acquisition Is Initialization) pattern -> TestSetup instance goes out of scope at the end of each test.
    impl Drop for TestSetup {
        fn drop(&mut self) {
            self.rt.block_on(async {
                self.clean_topology(None).await;
            });
        }
    }
}
