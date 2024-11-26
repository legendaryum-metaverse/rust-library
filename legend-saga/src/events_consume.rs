use crate::emitter::Emitter;
use crate::events::MicroserviceEvent;
use crate::my_delivery::MyDelivery;
use crate::nack::Nack;
use futures_lite::StreamExt;
use lapin::options::{BasicAckOptions, BasicConsumeOptions, BasicNackOptions, BasicQosOptions};
use lapin::types::{AMQPValue, FieldTable};
use lapin::Channel;
use serde::Deserialize;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::time::Duration;
use strum::IntoEnumIterator;
use tracing::{error, info};
use crate::connection::{RabbitMQClient, RabbitMQError};

#[derive(Clone)]
pub struct EventHandler {
    payload: HashMap<String, Value>,
    channel: EventsConsumeChannel,
}
impl EventHandler {
    pub fn parse_payload<T>(&self) -> Result<T, serde_json::Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        let json_value = serde_json::to_value(self.payload.clone())?;
        serde_json::from_value(json_value)
    }

    pub fn get_payload(&self) -> &HashMap<String, Value> {
        &self.payload
    }

    pub async fn ack(&self) -> Result<(), RabbitMQError> {
        self.channel.ack().await
    }

    pub async fn nack_with_delay(
        &self,
        delay: Duration,
        max_retries: i32,
    ) -> Result<(i32, Duration), RabbitMQError> {
        self.channel.nack.with_delay(delay, max_retries).await
    }

    pub async fn nack_with_fibonacci_strategy(
        &self,
        max_occurrence: i32,
        max_retries: i32,
    ) -> Result<(i32, Duration, i32), RabbitMQError> {
        self.channel
            .nack
            .with_fibonacci_strategy(max_occurrence, max_retries)
            .await
    }
}

impl RabbitMQClient {
    pub(crate) async fn consume_events(
        &self,
        queue_name: &str,
        emitter: Emitter<EventHandler, MicroserviceEvent>,
    ) -> Result<(), RabbitMQError> {
        let channel = self.events_channel.lock().await;
        channel.basic_qos(1, BasicQosOptions::default()).await?;

        let mut consumer = channel
            .basic_consume(
                queue_name,
                "event_consumer",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;

        // it needs to drop manually, next is an infinite loop
        drop(channel);

        while let Some(delivery) = consumer.next().await {
            match delivery {
                Ok(delivery) => {
                    if let Err(e) = self.handle_event(&delivery, &emitter, queue_name).await {
                        error!("Error handling event: {:?}", e);
                        let _ = delivery.nack(BasicNackOptions::default()).await;
                    }
                }
                Err(e) => {
                    error!("Error receiving message: {:?}", e);
                }
            }
        }
        Ok(())
    }

    async fn handle_event(
        &self,
        delivery: &lapin::message::Delivery,
        emitter: &Emitter<EventHandler, MicroserviceEvent>,
        queue_name: &str,
    ) -> Result<(), RabbitMQError> {
        let payload: HashMap<String, Value> = serde_json::from_slice(&delivery.data)?;

        let event_key =
            Self::find_event_values(&delivery.properties.headers().clone().unwrap_or_default())?;

        if event_key.len() > 1 {
            info!("More than one valid header, using the first one detected");
        }

        let event = &event_key[0];

        let channel = self.events_channel.lock().await;
        let delivery = MyDelivery::new(delivery);

        let response_channel =
            EventsConsumeChannel::new(channel.clone(), delivery, queue_name.to_string());

        let event_handler = EventHandler {
            payload,
            channel: response_channel,
        };

        emitter.emit(*event, event_handler).await;

        Ok(())
    }

    fn find_event_values(headers: &FieldTable) -> Result<Vec<MicroserviceEvent>, RabbitMQError> {
        let valid_events: HashSet<_> = MicroserviceEvent::iter().collect();

        let event_values: Vec<MicroserviceEvent> = headers
            .inner()
            .iter()
            .filter_map(|(_, value)| {
                if let AMQPValue::LongString(s) = value {
                    let event_str = s.to_string();
                    MicroserviceEvent::from_str(&event_str).ok()
                } else {
                    None
                }
            })
            .filter(|event| valid_events.contains(event))
            .collect();

        if event_values.is_empty() {
            Err(RabbitMQError::InvalidHeader)
        } else {
            Ok(event_values)
        }
    }
}

#[derive(Clone)]
struct EventsConsumeChannel {
    channel: Channel,
    delivery: MyDelivery,
    #[allow(dead_code)]
    queue_name: String,
    nack: Nack,
}

impl EventsConsumeChannel {
    fn new(channel: Channel, delivery: MyDelivery, queue_name: String) -> Self {
        Self {
            channel: channel.clone(),
            delivery: delivery.clone(),
            queue_name: queue_name.clone(),
            nack: Nack::new(channel, delivery, queue_name),
        }
    }

    async fn ack(&self) -> Result<(), RabbitMQError> {
        self.channel
            .basic_ack(self.delivery.delivery_tag, BasicAckOptions::default())
            .await
            .map_err(RabbitMQError::from)
    }
}

#[cfg(test)]
mod test_events {
    use super::*;
    use lapin::types::ShortString;

    #[test]
    fn test_find_event_values() {
        let mut headers = FieldTable::default();
        headers.insert(
            ShortString::from("event1"),
            AMQPValue::LongString("auth.deleted_user".into()),
        );
        headers.insert(
            ShortString::from("event2"),
            AMQPValue::LongString("invalid_event".into()),
        );
        headers.insert(
            ShortString::from("event3"),
            AMQPValue::LongString("social.new_user".into()),
        );

        let result = RabbitMQClient::find_event_values(&headers);
        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 2);
        assert!(events.contains(&MicroserviceEvent::AuthDeletedUser));
        assert!(events.contains(&MicroserviceEvent::SocialNewUser));
    }

    #[test]
    fn test_find_event_values_no_valid_events() {
        let mut headers = FieldTable::default();
        headers.insert(
            ShortString::from("event1"),
            AMQPValue::LongString("invalid_event1".into()),
        );
        headers.insert(
            ShortString::from("event2"),
            AMQPValue::LongString("invalid_event2".into()),
        );

        let result = RabbitMQClient::find_event_values(&headers);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RabbitMQError::InvalidHeader));
    }
}
