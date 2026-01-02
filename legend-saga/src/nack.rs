use crate::fibo::fibonacci;
use crate::my_delivery::MyDelivery;
use crate::queue_consumer_props::Exchange;
use lapin::options::{BasicNackOptions, BasicPublishOptions};
use lapin::types::{AMQPValue, FieldTable, ShortString};
use lapin::{BasicProperties, Channel};
use std::collections::BTreeMap;
use std::time::Duration;
use tracing::info;
use crate::connection::RabbitMQError;

#[derive(Clone)]
pub(crate) struct Nack {
    channel: Channel,
    delivery: MyDelivery,
    queue_name: String,
}
impl Nack {
    pub(crate) fn new(channel: Channel, delivery: MyDelivery, queue_name: String) -> Self {
        Self {
            channel,
            delivery,
            queue_name,
        }
    }
    pub(crate) async fn with_delay(
        &self,
        delay: Duration,
        max_retries: i32,
    ) -> Result<(i32, Duration), RabbitMQError> {
        self.channel
            .basic_nack(self.delivery.delivery_tag, BasicNackOptions::default())
            .await?;

        let count = self.calculate_retry_count();

        if count > max_retries as i64 {
            info!(
                "MAX NACK RETRIES REACHED: {} - NACKING {} - COUNT {}",
                max_retries, self.queue_name, count
            );
            return Ok((count as i32, delay));
        }
        let mut headers = self.delivery.headers.clone();
        headers.insert("x-retry-count".into(), AMQPValue::LongLongInt(count));

        self.publish_requeue(delay, headers).await?;
        Ok((count as i32, delay))
    }

    fn calculate_retry_count(&self) -> i64 {
        self.delivery
            .headers
            .inner()
            .get("x-retry-count")
            .and_then(|v| {
                if let AMQPValue::LongLongInt(n) = v {
                    Some(*n)
                } else {
                    None
                }
            })
            .unwrap_or(0)
            + 1
    }
    pub(crate) async fn with_fibonacci_strategy(
        &self,
        max_occurrence: i32,
        max_retries: i32,
    ) -> Result<(i32, Duration, i32), RabbitMQError> {
        self.channel
            .basic_nack(self.delivery.delivery_tag, BasicNackOptions::default())
            .await?;

        let count = self.calculate_retry_count();

        let occurrence = self
            .delivery
            .headers
            .inner()
            .get("x-occurrence")
            .and_then(|v| {
                if let AMQPValue::LongLongInt(n) = v {
                    Some(*n)
                } else {
                    None
                }
            })
            .unwrap_or(0);
        // the occurrence is reset to 0 to avoid large delay in the next nack
        let occurrence = if occurrence >= max_occurrence as i64 {
            1
        } else {
            occurrence + 1
        };

        let delay = Duration::from_secs(fibonacci(occurrence as usize) as u64);

        if count > max_retries as i64 {
            info!(
                "MAX NACK RETRIES REACHED: {} - NACKING {}",
                max_retries, self.queue_name
            );
            return Ok((count as i32, delay, occurrence as i32));
        }

        let mut headers = self.delivery.headers.clone();
        headers.insert("x-retry-count".into(), AMQPValue::LongLongInt(count));
        headers.insert("x-occurrence".into(), AMQPValue::LongLongInt(occurrence));

        self.publish_requeue(delay, headers).await?;
        Ok((count as i32, delay, occurrence as i32))
    }
    async fn publish_requeue(
        &self,
        delay: Duration,
        headers: FieldTable,
    ) -> Result<(), RabbitMQError> {
        let (exchange, routing_key, new_headers) =
            if self.delivery.exchange == ShortString::from(Exchange::MATCHING) {
                let mut new_map: BTreeMap<ShortString, AMQPValue> = headers.inner().clone();
                new_map.remove("all-micro");
                new_map.insert(
                    "micro".into(),
                    AMQPValue::LongString(self.queue_name.clone().into()),
                );
                (
                    Exchange::MATCHING_REQUEUE,
                    String::new(),
                    FieldTable::from(new_map),
                )
            } else {
                // is a saga event
                (
                    Exchange::REQUEUE,
                    format!("{}_routing_key", self.queue_name),
                    headers,
                )
            };

        self.channel
            .basic_publish(
                exchange,
                &routing_key,
                BasicPublishOptions::default(),
                &self.delivery.data.clone(),
                BasicProperties::default()
                    .with_expiration(delay.as_millis().to_string().into())
                    .with_headers(new_headers)
                    .with_app_id(self.delivery.app_id().clone().unwrap_or_default())
                    .with_message_id(self.delivery.message_id().clone().unwrap_or_default())
                    .with_delivery_mode(2), // persistent
            )
            .await?;

        Ok(())
    }
}
#[cfg(test)]
mod test_nack {
    use crate::events::{AuthLogoutUserPayload, MicroserviceEvent, SocialBlockChatPayload};
    use crate::test::setup::{Config, TestSetup};
    use std::sync::atomic::AtomicUsize;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::Barrier;
    use tokio::time::timeout;
    use crate::connection::{AvailableMicroservices, RabbitMQClient};

    /// Integration test, slow because of nack with fibo, min -> 1 sec
    #[test]
    fn nack_with_delay() {
        let setup = TestSetup::new(Some(Config {
            events: &[
                MicroserviceEvent::AuthLogoutUser,
                MicroserviceEvent::SocialBlockChat,
            ],
            microservice: AvailableMicroservices::Auth,
        }));
        setup.rt.block_on(async {
            // Connect to events
            let e = setup
                .client
                .connect_to_events()
                .await
                .expect("Failed to connect to events");

            let barrier = Arc::new(Barrier::new(3));
            let atomic_counter_logout = Arc::new(AtomicUsize::new(0));

            let c_barrier = barrier.clone();
            let c_atomic = atomic_counter_logout.clone();
            // 3 nacks and then ack!
            e.on_with_async_handler(MicroserviceEvent::AuthLogoutUser, move |handler| {
                let count = c_atomic.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let barrier = c_barrier.clone();

                async move {
                    if count == 3 {
                        let p: AuthLogoutUserPayload =
                            handler.parse_payload().expect("Failed to parse payload");
                        assert_eq!(p.user_id, "123");
                        handler.ack().await.expect("Failed to ack");
                        barrier.wait().await;
                        return;
                    }
                    handler
                        .nack_with_delay(Duration::from_millis(100), 30)
                        .await
                        .expect("Failed to nack");
                }
            })
            .await;

            let atomic_counter_social = Arc::new(AtomicUsize::new(0));
            let c_barrier = barrier.clone();
            let c_atomic = atomic_counter_social.clone();
            // 2 nacks and then ack!
            e.on_with_async_handler(MicroserviceEvent::SocialBlockChat, move |handler| {
                let count = c_atomic.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let barrier = c_barrier.clone();

                async move {

                    // Assert in consume and nack consume. En nack se tiene que pasar de nuevo estas props, chequear "publish_requeue"
                    let publisher_microservice = handler.publisher_microservice();
                    assert_eq!(publisher_microservice, "auth");
                    let event_id = handler.event_id();
                    assert!(uuid::Uuid::parse_str(event_id).is_ok());

                    if count == 2 {
                        let p: SocialBlockChatPayload =
                            handler.parse_payload().expect("Failed to parse payload");
                        assert_eq!(p.user_to_block_id, "blocked_user_456");
                        handler.ack().await.expect("Failed to ack");
                        barrier.wait().await;
                        return;
                    }
                    handler
                        .nack_with_fibonacci_strategy(10, 30)
                        .await
                        .expect("Failed to nack");
                }
            })
            .await;

            // Publish event
            RabbitMQClient::publish_event(AuthLogoutUserPayload {
                    user_id: "123".to_string(),
                })
                .await
                .expect("Failed to publish event");
            RabbitMQClient::publish_event(SocialBlockChatPayload {
                    user_id: "123".to_string(),
                    user_to_block_id: "blocked_user_456".to_string(),
                })
                .await
                .expect("Failed to publish event");

            timeout(Duration::from_secs(5), barrier.wait())
                .await
                .expect("Failed to wait for barrier");
            assert_eq!(
                atomic_counter_logout.load(std::sync::atomic::Ordering::SeqCst),
                4
            );
            assert_eq!(
                atomic_counter_social.load(std::sync::atomic::Ordering::SeqCst),
                3
            );
            //because fetch_add, returns and adds
        });
    }
}
