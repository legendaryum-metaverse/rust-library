use std::sync::Arc;
use std::time::Duration;
use lapin::{Channel, Connection};
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, EnumIter, EnumString};
use thiserror::Error;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, warn};
use crate::events::MicroserviceEvent;
use backoff::{Error as BackoffError, ExponentialBackoff};

#[derive(Debug, Clone, PartialEq, Eq, EnumString, AsRefStr, EnumIter, Serialize, Deserialize)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum AvailableMicroservices {
    TestImage,
    TestMint,
    Auth,
    Coins,
    Missions,
    Rankings,
    RapidMessaging,
    RoomCreator,
    RoomInventory,
    RoomSnapshot,
    SendEmail,
    Showcase,
    Social,
    SocialMediaRooms,
    Storage,
}

#[derive(Error, Debug)]
pub enum RabbitMQError {
    #[error("Connection error: {0}")]
    ConnectionError(#[from] lapin::Error),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Channel closed")]
    ChannelClosed,
    #[error("Backoff error: {0}")]
    BackoffError(String),
    #[error("Timeout error")]
    TimeoutError,
    #[error("Invalid header: no valid event key found")]
    InvalidHeader,
    #[error("Invalid event key: {0}")]
    InvalidEventKey(String),
    #[error("Invalid payload: {0}")]
    InvalidPayload(String),
}

#[derive(Debug, Error)]
pub enum HealthCheckError {
    #[error("Unhealthy {0}")]
    Unhealthy(String),
    #[error("Health check timed out after {0} milliseconds")]
    Timeout(u128),
}

pub struct RabbitMQClient {
    // RwLock for the connection because we expect many concurrent reads (status checks) and infrequent writes (reconnections).
    pub(crate) connection: Arc<RwLock<Connection>>,
    pub(crate) events_channel: Arc<Mutex<Channel>>,
    pub(crate) saga_channel: Arc<Mutex<Channel>>,
    pub(crate) events: &'static [MicroserviceEvent],
    pub(crate) microservice: AvailableMicroservices,
    rabbit_uri: String,
    reconnecting: Arc<Mutex<bool>>,
}

impl Clone for RabbitMQClient {
    fn clone(&self) -> Self {
        Self {
            events: self.events,
            microservice: self.microservice.clone(),
            connection: Arc::clone(&self.connection),
            events_channel: Arc::clone(&self.events_channel),
            saga_channel: Arc::clone(&self.saga_channel),
            rabbit_uri: self.rabbit_uri.clone(),
            reconnecting: Arc::clone(&self.reconnecting),
        }
    }
}

impl RabbitMQClient {
    pub async fn new(
        rabbit_uri: &str,
        microservice: AvailableMicroservices,
        events: Option<&'static [MicroserviceEvent]>,
    ) -> Result<Self, RabbitMQError> {
        let connection = Self::create_connection(rabbit_uri).await?;
        let events_channel = connection.create_channel().await?;
        events_channel
            .basic_qos(1, lapin::options::BasicQosOptions::default())
            .await?;

        let saga_channel = connection.create_channel().await?;
        saga_channel
            .basic_qos(1, lapin::options::BasicQosOptions::default())
            .await?;

        Ok(Self {
            microservice,
            events: events.unwrap_or(&[]),
            connection: Arc::new(RwLock::new(connection)),
            events_channel: Arc::new(Mutex::new(events_channel)),
            saga_channel: Arc::new(Mutex::new(saga_channel)),
            rabbit_uri: rabbit_uri.to_string(),
            reconnecting: Arc::new(Mutex::new(false)),
        })
    }
    pub fn print_init_message(&self) {
        info!(
            "\x1b[32m📡 Microservice: {:?} connected to Saga Command Emitter listening events: {:?}\x1b[0m",
            self.microservice, self.events
        );
    }

    /// health_check_with_reconnection tries to reconnect during 60s in the background,
    /// the timeout is for the "normal" health_check
    pub async fn health_check_with_reconnection(
        &self,
        timeout: Duration,
    ) -> Result<(), HealthCheckError> {
        let reconnecting = self.reconnecting.lock().await;
        if *reconnecting {
            return Err(HealthCheckError::Unhealthy(
                "Reconnecting the server...".to_string(),
            ));
        }
        drop(reconnecting);
        let hc = self.health_check(timeout).await;
        if hc.is_err() {
            let c_reconnecting = self.reconnecting.clone();
            let client = self.clone();
            tokio::spawn(async move {
                let mut reconnecting = c_reconnecting.lock().await;
                *reconnecting = true;
                drop(reconnecting);
                if let Err(e) = client.reconnect().await {
                    error!("Error reconnecting: {:?}", e);
                    let mut reconnecting = c_reconnecting.lock().await;
                    *reconnecting = false;
                }
            });
        }
        hc
    }

    /// health_check checks the health of the RabbitMQ connection, events channel, and saga channel.
    /// timeout is the maximum time to wait for the health check to complete. ie: the channel can be locked
    pub async fn health_check(&self, timeout: Duration) -> Result<(), HealthCheckError> {
        let health_check = async {
            // also possible with try_join_all from futures crate
            futures_lite::future::try_zip(
                self.check_connection_health(),
                futures_lite::future::try_zip(
                    self.check_events_channel_health(),
                    self.check_saga_channel_health(),
                ),
            )
                .await?;
            Ok(())
        };

        tokio::time::timeout(timeout, health_check)
            .await
            .map_err(|_| HealthCheckError::Timeout(timeout.as_millis()))?
    }

    async fn check_connection_health(&self) -> Result<(), HealthCheckError> {
        let conn = self.connection.read().await;
        if !conn.status().connected() {
            return Err(HealthCheckError::Unhealthy("Connection".to_string()));
        }
        Ok(())
    }

    async fn check_events_channel_health(&self) -> Result<(), HealthCheckError> {
        let chan = self.events_channel.lock().await;
        if !chan.status().connected() {
            return Err(HealthCheckError::Unhealthy("Events Channel".to_string()));
        }
        Ok(())
    }

    async fn check_saga_channel_health(&self) -> Result<(), HealthCheckError> {
        let chan = self.saga_channel.lock().await;
        if !chan.status().connected() {
            return Err(HealthCheckError::Unhealthy("Saga Channel".to_string()));
        }
        Ok(())
    }

    async fn create_connection(addr: &str) -> Result<Connection, RabbitMQError> {
        let backoff = ExponentialBackoff {
            max_elapsed_time: Some(Duration::from_secs(60)),
            ..Default::default()
        };

        backoff::future::retry(backoff, || async {
            info!("Attempting to connect to RabbitMQ");
            Connection::connect(addr, Default::default())
                .await
                .map_err(BackoffError::transient)
        })
            .await
            .map_err(|e| RabbitMQError::BackoffError(e.to_string()))
    }

    pub async fn reconnect(&self) -> Result<(), RabbitMQError> {
        warn!("Attempting to reconnect to RabbitMQ");

        let new_connection = Self::create_connection(&self.rabbit_uri).await?;
        let events_channel = new_connection.create_channel().await?;
        let saga_channel = new_connection.create_channel().await?;

        // Update the connection
        let mut conn_write = self.connection.write().await;
        *conn_write = new_connection;
        drop(conn_write); // Explicitly drop the write lock

        // Update the channel
        let mut channel = self.events_channel.lock().await;
        *channel = events_channel;

        let mut channel = self.saga_channel.lock().await;
        *channel = saga_channel;

        info!("Successfully reconnected to RabbitMQ");
        let mut reconnecting = self.reconnecting.lock().await;
        *reconnecting = false;
        Ok(())
    }

    pub async fn cleanup(&self) {
        debug!("Cleaning up RabbitMQ client resources");
        let channel = self.events_channel.lock().await;
        if let Err(e) = channel.close(0, "Cleanup").await {
            warn!("Error closing events_channel: {:?}", e);
        }
        let channel = self.saga_channel.lock().await;
        if let Err(e) = channel.close(0, "Cleanup").await {
            warn!("Error closing saga_channel: {:?}", e);
        }

        let conn = self.connection.read().await;
        if let Err(e) = conn.close(0, "Cleanup").await {
            warn!("Error closing connection: {:?}", e);
        }
        debug!("RabbitMQ client resources cleaned up");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::setup::{TestSetup, TEST_QUEUE};
    use futures_lite::StreamExt;
    use lapin::options::{BasicConsumeOptions, QueueDeclareOptions};
    use lapin::BasicProperties;
    use std::time::Duration;

    mod health {
        use super::*;
        #[test]
        fn test_health_check() {
            let setup = TestSetup::new(None);

            setup.rt.block_on(async {
                let res = setup.client.health_check(Duration::from_millis(100)).await;
                assert!(res.is_ok());
            });
            // Replace with RAII pattern
            // setup.cleanup();
        }
        #[test]
        fn test_health_check_timeout() {
            let setup = TestSetup::new(None);

            setup.rt.block_on(async {
                let _chan_lock = setup.client.saga_channel.lock().await; // blocking
                let res = setup.client.health_check(Duration::from_millis(100)).await;
                assert!(res.is_err());
                assert_eq!(
                    res.unwrap_err().to_string(),
                    "Health check timed out after 100 milliseconds"
                );
            });
        }
        #[test]
        fn test_health_check_saga_chan() {
            let setup = TestSetup::new(None);

            setup.rt.block_on(async {
                let result = setup.client.check_saga_channel_health().await;
                assert!(result.is_ok());
            });
        }
        #[test]
        fn test_healthcheck_saga_chan_close() {
            let setup = TestSetup::new(None);
            setup.rt.block_on(async {
                let chan = setup.client.saga_channel.lock().await; // blocking
                chan.close(0, "Test disconnect")
                    .await
                    .expect("Failed to close channel");
                drop(chan); // otherwise is test_health_check_timeout test

                let result = setup.client.check_saga_channel_health().await;
                assert!(result.is_err());
                assert_eq!(result.unwrap_err().to_string(), "Unhealthy Saga Channel");

                let result = setup.client.health_check(Duration::from_millis(200)).await;
                assert!(result.is_err());
                assert_eq!(result.unwrap_err().to_string(), "Unhealthy Saga Channel");
            });
        }
        #[test]
        fn test_health_check_events_chan() {
            let setup = TestSetup::new(None);

            setup.rt.block_on(async {
                let result = setup.client.check_events_channel_health().await;
                assert!(result.is_ok());
            });
        }
        #[test]
        fn test_healthcheck_events_chan_close() {
            let setup = TestSetup::new(None);
            setup.rt.block_on(async {
                let chan = setup.client.events_channel.lock().await; // blocking
                chan.close(0, "Test disconnect")
                    .await
                    .expect("Failed to close channel");
                drop(chan); // otherwise is test_health_check_timeout test

                let result = setup.client.check_events_channel_health().await;
                assert!(result.is_err());
                assert_eq!(result.unwrap_err().to_string(), "Unhealthy Events Channel");

                let result = setup.client.health_check(Duration::from_millis(200)).await;
                assert!(result.is_err());
                assert_eq!(result.unwrap_err().to_string(), "Unhealthy Events Channel");
            });
        }
        #[test]
        fn test_health_check_connection() {
            let setup = TestSetup::new(None);

            setup.rt.block_on(async {
                let result = setup.client.check_connection_health().await;
                assert!(result.is_ok());
            });
        }
        #[test]
        fn test_health_check_connection_close() {
            let setup = TestSetup::new(None);
            setup.rt.block_on(async {
                let conn_lock = setup.client.connection.write().await;
                conn_lock
                    .close(0, "Test disconnect")
                    .await
                    .expect("Failed to close connection");
                drop(conn_lock); // otherwise is test_health_check_timeout test

                let result = setup.client.check_connection_health().await;
                assert!(result.is_err());
                assert_eq!(result.unwrap_err().to_string(), "Unhealthy Connection");

                let result = setup.client.health_check(Duration::from_millis(200)).await;
                assert!(result.is_err());
                assert_eq!(result.unwrap_err().to_string(), "Unhealthy Connection");
            });
        }
    }
    #[test]
    fn test_reconnection() {
        let setup = TestSetup::new(None);

        setup.rt.block_on(async {
            // Step 1: Check initial health
            let healthy = setup.client.health_check(Duration::from_millis(200)).await;
            assert!(
                healthy.is_ok(),
                "RabbitMQ connection should be healthy before reconnection"
            );

            // Step 2: Simulate a connection drop by manually closing the connection
            {
                let conn = setup.client.connection.write().await;
                conn.close(0, "Test disconnect")
                    .await
                    .expect("Failed to close connection");
            }

            // Step 3: Ensure connection is unhealthy
            let healthy = setup.client.health_check(Duration::from_millis(200)).await;
            assert!(
                healthy.is_err(),
                "RabbitMQ connection should be healthy before reconnection"
            );

            // Step 4: Trigger reconnection and check health again
            setup
                .client
                .reconnect()
                .await
                .expect("Reconnection should succeed");

            let healthy = setup.client.health_check(Duration::from_millis(200)).await;
            assert!(
                healthy.is_ok(),
                "RabbitMQ connection should be healthy before reconnection"
            );
        });
    }

    #[test]
    fn test_declare_queue() {
        let setup = TestSetup::new(None);

        setup.rt.block_on(async {
            assert!(
                setup
                    .client
                    .declare_queue(
                        TEST_QUEUE,
                        QueueDeclareOptions::default(),
                        Default::default()
                    )
                    .await
                    .is_ok(),
                "Should be able to declare a queue"
            );
        });
    }

    #[test]
    fn test_publish_and_consume() {
        let setup = TestSetup::new(None);
        setup.rt.block_on(async {
            setup
                .client
                .declare_queue(TEST_QUEUE, Default::default(), Default::default())
                .await
                .expect("Failed to declare queue");

            #[derive(Debug, Serialize, Deserialize, PartialEq)]
            struct TestMessage {
                content: String,
            }

            let test_message = TestMessage {
                content: "Test message".to_string(),
            };

            let properties = BasicProperties::default()
                .with_delivery_mode(2)
                .with_content_type("application/json".into());

            setup
                .client
                .publish_message(TEST_QUEUE, &test_message, properties)
                .await
                .expect("Failed to publish message");

            let mut consumer = setup
                .client
                .consume_messages::<TestMessage>(TEST_QUEUE, BasicConsumeOptions::default())
                .await
                .expect("Failed to create consumer");

            let received_message = tokio::time::timeout(Duration::from_secs(5), consumer.next())
                .await
                .expect("Timed out waiting for message")
                .expect("Failed to receive message")
                .expect("Error in received message");

            assert_eq!(
                received_message, test_message,
                "Received message should match sent message"
            );
        });
    }

    #[test]
    fn test_multiple_message_publish_and_consume() {
        let setup = TestSetup::new(None);
        setup.rt.block_on(async {
            setup
                .client
                .declare_queue(TEST_QUEUE, Default::default(), Default::default())
                .await
                .expect("Failed to declare queue");

            #[derive(Debug, Serialize, Deserialize, PartialEq)]
            struct TestMessage {
                content: String,
            }

            let messages = vec![
                TestMessage {
                    content: "Message 1".to_string(),
                },
                TestMessage {
                    content: "Message 2".to_string(),
                },
                TestMessage {
                    content: "Message 3".to_string(),
                },
            ];

            let properties = BasicProperties::default()
                .with_delivery_mode(2)
                .with_content_type("application/json".into());

            // Step 1: Publish multiple messages
            for message in &messages {
                setup
                    .client
                    .publish_message(TEST_QUEUE, message, properties.clone())
                    .await
                    .expect("Failed to publish message");
            }

            // Step 2: Consume the messages and verify the order
            let mut consumer = setup
                .client
                .consume_messages::<TestMessage>(TEST_QUEUE, BasicConsumeOptions::default())
                .await
                .expect("Failed to create consumer");

            for expected_message in &messages {
                let received_message =
                    tokio::time::timeout(Duration::from_secs(5), consumer.next())
                        .await
                        .expect("Timed out waiting for message")
                        .expect("Failed to receive message")
                        .expect("Error in received message");

                assert_eq!(
                    received_message, *expected_message,
                    "Received message should match expected message"
                );
            }
        });
    }
    #[test]
    fn test_reconnection_during_message_consumption() {
        let setup = TestSetup::new(None);
        setup.rt.block_on(async {
            setup
                .client
                .declare_queue(TEST_QUEUE, Default::default(), Default::default())
                .await
                .expect("Failed to declare queue");

            #[derive(Debug, Serialize, Deserialize, PartialEq)]
            struct TestMessage {
                content: String,
            }

            let message = TestMessage {
                content: "Message before reconnect".to_string(),
            };

            let properties = BasicProperties::default()
                .with_delivery_mode(2)
                .with_content_type("application/json".into());

            // Step 1: Publish a message
            setup
                .client
                .publish_message(TEST_QUEUE, &message, properties.clone())
                .await
                .expect("Failed to publish message");

            // Step 2: Consume the message and trigger reconnection in between
            let mut consumer = setup
                .client
                .consume_messages::<TestMessage>(TEST_QUEUE, BasicConsumeOptions::default())
                .await
                .expect("Failed to create consumer");

            // Step 3: Simulate a connection drop while consuming
            {
                let conn = setup.client.connection.write().await;
                conn.close(0, "Test disconnect")
                    .await
                    .expect("Failed to close connection");
            }

            // Step 4: Trigger reconnection
            setup
                .client
                .reconnect()
                .await
                .expect("Reconnection should succeed");

            // Step 5: Ensure the remaining message can still be consumed
            let received_message = tokio::time::timeout(Duration::from_secs(5), consumer.next())
                .await
                .expect("Timed out waiting for message")
                .expect("Failed to receive message")
                .expect("Error in received message");

            assert_eq!(
                received_message, message,
                "Received message should match expected message"
            );
        });
    }

    #[test]
    fn test_concurrent_operations() {
        let setup = TestSetup::new(None);
        setup.rt.block_on(async {
            setup
                .client
                .declare_queue(TEST_QUEUE, Default::default(), Default::default())
                .await
                .expect("Failed to declare queue");

            #[derive(Debug, Serialize, Deserialize, PartialEq)]
            struct TestMessage {
                content: String,
            }

            let properties = BasicProperties::default()
                .with_delivery_mode(2) // Persistent
                .with_content_type("application/json".into());

            let num_messages = 100; // Number of concurrent messages to publish
            let mut publish_futures = vec![];

            for i in 0..num_messages {
                let client_clone = setup.client.clone();
                let properties_clone = properties.clone();
                let message = TestMessage {
                    content: format!("Concurrent message {}", i),
                };

                publish_futures.push(tokio::spawn(async move {
                    client_clone
                        .publish_message(TEST_QUEUE, &message, properties_clone)
                        .await
                        .expect("Failed to publish message");
                }));
            }

            // Await all publish operations
            for future in publish_futures {
                future.await.unwrap();
            }

            // Now consume the messages and verify
            let mut consumer = setup
                .client
                .consume_messages::<TestMessage>(TEST_QUEUE, BasicConsumeOptions::default())
                .await
                .expect("Failed to create consumer");

            let mut received_messages = vec![];
            for _ in 0..num_messages {
                let received_message =
                    tokio::time::timeout(Duration::from_secs(5), consumer.next())
                        .await
                        .expect("Timed out waiting for message")
                        .expect("Failed to receive message")
                        .expect("Error in received message");

                received_messages.push(received_message);
            }

            assert_eq!(
                received_messages.len(),
                num_messages,
                "Should receive all published messages"
            );
        });
    }
}
