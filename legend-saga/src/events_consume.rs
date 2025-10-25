use crate::emitter::Emitter;
use crate::events::{
    AuditDeadLetterPayload, AuditProcessedPayload, AuditReceivedPayload, MicroserviceEvent,
};
use crate::my_delivery::MyDelivery;
use crate::nack::Nack;
use crate::queue_consumer_props::Queue;
use futures_lite::StreamExt;
use lapin::options::{BasicAckOptions, BasicConsumeOptions, BasicNackOptions};
use lapin::types::{AMQPValue, FieldTable};
use lapin::Channel;
use serde::Deserialize;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use strum::IntoEnumIterator;
use tracing::{error, info, warn};
use crate::connection::{RabbitMQClient, RabbitMQError};
use uuid::Uuid;

#[derive(Clone)]
pub struct EventHandler {
    payload: HashMap<String, Value>,
    channel: EventsConsumeChannel,
    microservice: String,
    processed_event: String,
    publisher_microservice: String,
    event_id: String,
}
impl EventHandler {

    pub fn publisher_microservice(&self) -> &String {
        &self.publisher_microservice
    }

    pub fn event_id(&self) -> &String {
        &self.event_id
    }
    
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
        // First, ack the original message
        self.channel.ack().await?;

        // Then emit audit.processed event automatically
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let audit_payload = AuditProcessedPayload {
            publisher_microservice: self.publisher_microservice.clone(),
            processor_microservice: self.microservice.clone(),
            processed_event: self.processed_event.clone(),
            processed_at: timestamp,
            queue_name: self.channel.queue_name.clone(),
            event_id: self.event_id.clone(),
        };

        // Emit the audit event using the new direct exchange method
        tokio::spawn(async move {
            if let Err(e) = RabbitMQClient::publish_audit_event(audit_payload).await {
                error!("Failed to emit audit.processed event: {:?}", e);
            }
        });

        Ok(())
    }

    pub async fn nack_with_delay(
        &self,
        delay: Duration,
        max_retries: i32,
    ) -> Result<(i32, Duration), RabbitMQError> {
        let result = self.channel.nack.with_delay(delay, max_retries).await?;

        // Emit audit.dead_letter event automatically
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let audit_payload = AuditDeadLetterPayload {
            publisher_microservice: self.publisher_microservice.clone(),
            rejector_microservice: self.microservice.clone(),
            rejected_event: self.processed_event.clone(),
            rejected_at: timestamp,
            queue_name: self.channel.queue_name.clone(),
            rejection_reason: "delay".to_string(),
            retry_count: Some(result.0 as u32),
            event_id: self.event_id.clone(),
        };

        // Emit the audit event (don't fail if audit fails)
        tokio::spawn(async move {
            if let Err(e) = RabbitMQClient::publish_audit_event(audit_payload).await {
                error!("Failed to emit audit.dead_letter event: {:?}", e);
            }
        });

        Ok(result)
    }

    pub async fn nack_with_fibonacci_strategy(
        &self,
        max_occurrence: i32,
        max_retries: i32,
    ) -> Result<(i32, Duration, i32), RabbitMQError> {
        let result = self
            .channel
            .nack
            .with_fibonacci_strategy(max_occurrence, max_retries)
            .await?;

        // Emit audit.dead_letter event automatically
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let audit_payload = AuditDeadLetterPayload {
            publisher_microservice: self.publisher_microservice.clone(),
            rejector_microservice: self.microservice.clone(),
            rejected_event: self.processed_event.clone(),
            rejected_at: timestamp,
            queue_name: self.channel.queue_name.clone(),
            rejection_reason: "fibonacci_strategy".to_string(),
            retry_count: Some(result.0 as u32),
            event_id: self.event_id.clone(),
        };

        // Emit the audit event (don't fail if audit fails)
        tokio::spawn(async move {
            if let Err(e) = RabbitMQClient::publish_audit_event(audit_payload).await {
                error!("Failed to emit audit.dead_letter event: {:?}", e);
            }
        });

        Ok(result)
    }
}

impl RabbitMQClient {
    pub(crate) async fn consume_events(
        &self,
        queue_name: &str,
        emitter: Emitter<EventHandler, MicroserviceEvent>,
    ) -> Result<(), RabbitMQError> {
        let channel = self.events_channel.lock().await;

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

        let publisher_microservice = delivery.properties.app_id()
            .as_ref()
            .map(|id| id.to_string())
            .unwrap_or_else(|| {
                warn!("Message is missing app_id (publisher_microservice), defaulting to 'unknown'");
                "unknown".to_string()
            });

        let event_id = delivery.properties.message_id()
            .as_ref()
            .map(|id| id.to_string())
            .unwrap_or_else(|| {
                warn!("Message is missing message_id, generating a new UUID v7 for event_id");
                Uuid::now_v7().to_string()
            });

        let channel = self.events_channel.lock().await;
        let delivery = MyDelivery::new(delivery).with_app_id(publisher_microservice.clone().into()).with_message_id(event_id.clone().into());

        let response_channel =
            EventsConsumeChannel::new(channel.clone(), delivery, queue_name.to_string());

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let audit_payload = AuditReceivedPayload {
            publisher_microservice: publisher_microservice.clone(),
            receiver_microservice: self.microservice.as_ref().to_string(),
            received_event: event.as_ref().to_string(),
            received_at: timestamp,
            queue_name: queue_name.to_string(),
            event_id: event_id.clone(),
        };

        // Emit the audit.received event (don't fail the main flow if audit fails)
        tokio::spawn(async move {
            if let Err(e) = RabbitMQClient::publish_audit_event(audit_payload).await {
                error!("Failed to emit audit.received event: {:?}", e);
            }
        });

        let event_handler = EventHandler {
            payload,
            channel: response_channel,
            microservice: self.microservice.as_ref().to_string(),
            processed_event: event.as_ref().to_string(),
            publisher_microservice,
            event_id,
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

    // ========== AUDIT CONSUMER METHODS ==========

    /// Consumes audit.received events from dedicated queue
    pub(crate) async fn consume_audit_received_events(
        &self,
        emitter: Emitter<AuditHandler, MicroserviceEvent>,
    ) -> Result<(), RabbitMQError> {
        let channel = self.events_channel.lock().await;

        let mut consumer = channel
            .basic_consume(
                Queue::AUDIT_RECEIVED_COMMANDS,
                "audit_received_consumer",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;

        drop(channel);

        while let Some(delivery) = consumer.next().await {
            match delivery {
                Ok(delivery) => {
                    if let Err(e) = self.handle_audit_event(&delivery, &emitter, Queue::AUDIT_RECEIVED_COMMANDS).await {
                        error!("Error handling audit.received event: {:?}", e);
                        let _ = delivery.nack(BasicNackOptions::default()).await;
                    }
                }
                Err(e) => {
                    error!("Error receiving audit.received message: {:?}", e);
                }
            }
        }
        Ok(())
    }

    /// Consumes audit.processed events from dedicated queue
    pub(crate) async fn consume_audit_processed_events(
        &self,
        emitter: Emitter<AuditHandler, MicroserviceEvent>,
    ) -> Result<(), RabbitMQError> {
        let channel = self.events_channel.lock().await;

        let mut consumer = channel
            .basic_consume(
                Queue::AUDIT_PROCESSED_COMMANDS,
                "audit_processed_consumer",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;

        drop(channel);

        while let Some(delivery) = consumer.next().await {
            match delivery {
                Ok(delivery) => {
                    if let Err(e) = self.handle_audit_event(&delivery, &emitter, Queue::AUDIT_PROCESSED_COMMANDS).await {
                        error!("Error handling audit.processed event: {:?}", e);
                        let _ = delivery.nack(BasicNackOptions::default()).await;
                    }
                }
                Err(e) => {
                    error!("Error receiving audit.processed message: {:?}", e);
                }
            }
        }
        Ok(())
    }

    /// Consumes audit.dead_letter events from dedicated queue
    pub(crate) async fn consume_audit_dead_letter_events(
        &self,
        emitter: Emitter<AuditHandler, MicroserviceEvent>,
    ) -> Result<(), RabbitMQError> {
        let channel = self.events_channel.lock().await;

        let mut consumer = channel
            .basic_consume(
                Queue::AUDIT_DEAD_LETTER_COMMANDS,
                "audit_dead_letter_consumer",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;

        drop(channel);

        while let Some(delivery) = consumer.next().await {
            match delivery {
                Ok(delivery) => {
                    if let Err(e) = self.handle_audit_event(&delivery, &emitter, Queue::AUDIT_DEAD_LETTER_COMMANDS).await {
                        error!("Error handling audit.dead_letter event: {:?}", e);
                        let _ = delivery.nack(BasicNackOptions::default()).await;
                    }
                }
                Err(e) => {
                    error!("Error receiving audit.dead_letter message: {:?}", e);
                }
            }
        }
        Ok(())
    }

    /// Consumes audit.published events from dedicated queue
    pub(crate) async fn consume_audit_published_events(
        &self,
        emitter: Emitter<AuditHandler, MicroserviceEvent>,
    ) -> Result<(), RabbitMQError> {
        let channel = self.events_channel.lock().await;

        let mut consumer = channel
            .basic_consume(
                Queue::AUDIT_PUBLISHED_COMMANDS,
                "audit_published_consumer",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;

        drop(channel);

        while let Some(delivery) = consumer.next().await {
            match delivery {
                Ok(delivery) => {
                    if let Err(e) = self.handle_audit_event(&delivery, &emitter, Queue::AUDIT_PUBLISHED_COMMANDS).await {
                        error!("Error handling audit.published event: {:?}", e);
                        let _ = delivery.nack(BasicNackOptions::default()).await;
                    }
                }
                Err(e) => {
                    error!("Error receiving audit.published message: {:?}", e);
                }
            }
        }
        Ok(())
    }

    /// Handles audit events for the audit microservice
    async fn handle_audit_event(
        &self,
        delivery: &lapin::message::Delivery,
        emitter: &Emitter<AuditHandler, MicroserviceEvent>,
        queue_name: &str,
    ) -> Result<(), RabbitMQError> {
        let payload: HashMap<String, Value> = serde_json::from_slice(&delivery.data)?;

        // For audit events, we determine the event type from the routing key or queue name
        let event = match queue_name {
            Queue::AUDIT_PUBLISHED_COMMANDS => MicroserviceEvent::AuditPublished,
            Queue::AUDIT_RECEIVED_COMMANDS => MicroserviceEvent::AuditReceived,
            Queue::AUDIT_PROCESSED_COMMANDS => MicroserviceEvent::AuditProcessed,
            Queue::AUDIT_DEAD_LETTER_COMMANDS => MicroserviceEvent::AuditDeadLetter,
            _ => return Err(RabbitMQError::InvalidHeader),
        };

        let channel = self.events_channel.lock().await;
        let delivery = MyDelivery::new(delivery);

        let response_channel =
            EventsConsumeChannel::new(channel.clone(), delivery, queue_name.to_string());

        let audit_handler = AuditHandler {
            payload,
            channel: response_channel,
        };

        emitter.emit(event, audit_handler).await;

        Ok(())
    }
}

/// AuditHandler - specialized handler for audit events that doesn't emit recursive audits
/// This prevents the audit microservice from auditing its own audit processing
#[derive(Clone)]
pub struct AuditHandler {
    payload: HashMap<String, Value>,
    channel: EventsConsumeChannel,
}

impl AuditHandler {
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

    /// Audit-specific ack that doesn't emit recursive audit events
    pub async fn audit_ack(&self) -> Result<(), RabbitMQError> {
        self.channel.ack().await
    }

    /// Standard ack method - doesn't emit recursive audit events for AuditHandler
    pub async fn ack(&self) -> Result<(), RabbitMQError> {
        self.channel.ack().await
    }

    /// Nack with delay - no audit emission for audit handler
    /// Note: In future versions, we might want to consider auditing nacks from audit service
    pub async fn nack_with_delay(
        &self,
        delay: Duration,
        max_retries: i32,
    ) -> Result<(i32, Duration), RabbitMQError> {
        self.channel.nack.with_delay(delay, max_retries).await
    }

    /// Nack with fibonacci strategy - no audit emission for audit handler
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
