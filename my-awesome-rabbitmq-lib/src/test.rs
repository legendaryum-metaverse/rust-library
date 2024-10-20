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

    use lapin::types::FieldTable;
    use lapin::BasicProperties;
    use rand::distributions::{Distribution, Standard};
    use rand::Rng;
    use serde::de::DeserializeOwned;
    use serde::Serialize;
    use std::env;

    use crate::connection::{AvailableMicroservices, RabbitMQClient, RabbitMQError};
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
        async fn delete_queue(&self, queue_name: &str) -> Result<(), RabbitMQError> {
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

    impl Distribution<AvailableMicroservices> for Standard {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> AvailableMicroservices {
            use strum::IntoEnumIterator;
            AvailableMicroservices::iter()
                .nth(rng.gen_range(0..AvailableMicroservices::iter().count()))
                .unwrap()
        }
    }

    pub fn random_microservice() -> AvailableMicroservices {
        let mut rng = rand::thread_rng();
        rng.gen()
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
    }

    // Rust's RAII (Resource Acquisition Is Initialization) pattern -> TestSetup instance goes out of scope at the end of each test.
    impl Drop for TestSetup {
        fn drop(&mut self) {
            self.rt.block_on(async {
                let conn = self.client.connection.read().await;
                if !conn.status().connected() {
                    // must be the health check tests
                    return;
                }
                let delete_channel = conn.create_channel().await.unwrap();
                let t = conn.topology();
                for queue in t.queues {
                    debug!("DELETING QUEUE: {}", queue.name.to_string());
                    delete_channel
                        .queue_delete(&queue.name.to_string(), QueueDeleteOptions::default())
                        .await
                        .unwrap();
                }

                // delete all exchanges
                for exchange in t.exchanges {
                    debug!("DELETING EXCHANGE: {}", exchange.name.to_string());
                    delete_channel
                        .exchange_delete(&exchange.name.to_string(), Default::default())
                        .await
                        .unwrap();
                }
                debug!("RESTORED TOPOLOGY");
            });
        }
    }
}
