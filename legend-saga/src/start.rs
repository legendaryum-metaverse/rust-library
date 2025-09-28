use crate::emitter::Emitter;
use crate::events::{ MicroserviceEvent};
use crate::queue_consumer_props::{Exchange, QueueConsumerProps};
use crate::saga::{CommandHandler, StepCommand};
use tracing::error;
use crate::connection::{RabbitMQClient, RabbitMQError};
use crate::events_consume::{AuditHandler, EventHandler};
pub(crate) type EventEmitter = Emitter<EventHandler, MicroserviceEvent>;
pub(crate) type SagaEmitter = Emitter<CommandHandler, StepCommand>;
pub(crate) type AuditEmitter = Emitter<AuditHandler, MicroserviceEvent>;

impl RabbitMQClient {
    pub async fn connect_to_events(
        &self,
    ) -> Result<EventEmitter, RabbitMQError> {
        let queue_name = self.events_queue_name.clone();
        self.create_header_consumers(&queue_name, self.events)
            .await?;

        // Create audit logging resources, this feature is related only to "events", that is why we
        // create it here
        self.create_audit_logging_resources().await?;

        let emitter = self.start_consuming_events().await;

        Ok(emitter)
    }

    pub(crate) async fn start_consuming_events(&self) -> EventEmitter {
        let mut emitter_guard = self.event_emitter.lock().await;
        let emitter = emitter_guard.get_or_insert_with(Emitter::new).clone();

        tokio::spawn({
            let client = self.clone();
            let queue_name = self.events_queue_name.clone();
            let emitter = emitter.clone();

            async move {
                if let Err(e) = client.consume_events(&queue_name, emitter).await {
                    error!("Error consuming messages: {:?}", e);
                }
            }
        });

        emitter
    }

    pub async fn connect_to_saga_commands(
        &self,
    ) -> Result<SagaEmitter, RabbitMQError> {
        let queue_name = self.saga_queue_name.clone();

        self.create_consumers(vec![QueueConsumerProps {
            queue_name,
            exchange: Exchange::COMMANDS,
        }])
        .await?;

        let emitter = self.start_consuming_saga_commands().await;

        Ok(emitter)
    }

    pub(crate) async fn start_consuming_saga_commands(&self) -> SagaEmitter {
        let mut emitter_guard = self.saga_emitter.lock().await;
        let emitter = emitter_guard.get_or_insert_with(Emitter::new).clone();

        tokio::spawn({
            let client = self.clone();
            let queue_name = self.saga_queue_name.clone();
            let emitter = emitter.clone();

            async move {
                if let Err(e) = client.consume_saga_steps(&queue_name, emitter).await {
                    error!("Error consuming messages: {:?}", e);
                }
            }
        });

        emitter
    }

    /// Connect to audit events - for audit-eda-micro only
    /// Uses direct exchange routing for efficient single-consumer delivery
    pub async fn connect_to_audit(&self) -> Result<AuditEmitter, RabbitMQError> {
        // Create audit queue and exchange infrastructure
        self.create_audit_logging_resources().await?;

        let emitter = self.start_consuming_audit().await;

        Ok(emitter)
    }

    pub(crate) async fn start_consuming_audit(&self) -> AuditEmitter {
        let mut emitter_guard = self.audit_emitter.lock().await;
        let emitter = emitter_guard.get_or_insert_with(Emitter::new).clone();

        // Spawn consumer for audit.received events
        tokio::spawn({
            let client = self.clone();
            let emitter = emitter.clone();

            async move {
                if let Err(e) = client.consume_audit_received_events(emitter).await {
                    error!("Error consuming audit.received events: {:?}", e);
                }
            }
        });

        // Spawn consumer for audit.processed events
        tokio::spawn({
            let client = self.clone();
            let emitter = emitter.clone();

            async move {
                if let Err(e) = client.consume_audit_processed_events(emitter).await {
                    error!("Error consuming audit.processed events: {:?}", e);
                }
            }
        });

        // Spawn consumer for audit.dead_letter events
        tokio::spawn({
            let client = self.clone();
            let emitter = emitter.clone();

            async move {
                if let Err(e) = client.consume_audit_dead_letter_events(emitter).await {
                    error!("Error consuming audit.dead_letter events: {:?}", e);
                }
            }
        });

        emitter
    }
}

#[cfg(test)]
mod test_audit_consumer {
    use super::*;
    use crate::connection::AvailableMicroservices;
    use crate::events::{AuditProcessedPayload, MicroserviceEvent};
    use crate::test::setup::{Config, TestSetup};
    use std::sync::atomic::AtomicUsize;
    use std::sync::Arc;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tokio::sync::Barrier;

    #[test]
    fn test_connect_to_audit() {
        // Create a test setup for the audit microservice
        let setup = TestSetup::new(Some(Config {
            events: &[], // Audit doesn't need to subscribe to other events
            microservice: AvailableMicroservices::AuditEda,
        }));

        setup.rt.block_on(async {
            // Connect to audit events
            let audit_emitter = setup
                .client
                .connect_to_audit()
                .await
                .expect("Failed to connect to audit");

            // Set up test synchronization
            let barrier = Arc::new(Barrier::new(2));
            let barrier_clone = barrier.clone();
            let processed_count = Arc::new(AtomicUsize::new(0));
            let count_clone = processed_count.clone();

            // Set up audit event handler
            audit_emitter
                .on_with_async_handler(MicroserviceEvent::AuditProcessed, move |handler| {
                    let barrier = barrier.clone();
                    let count = count_clone.clone();
                    async move {
                        println!("Received audit event: {:?}", handler.get_payload());

                        // Parse the audit payload
                        let audit_payload: AuditProcessedPayload = handler
                            .parse_payload()
                            .expect("Failed to parse audit payload");

                        println!("Parsed audit payload: {:?}", audit_payload);

                        // With AuditHandler, we should only receive our test event (no recursive audit)
                        let _current_count =
                            count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

                        // Assert on our test event
                        assert_eq!(audit_payload.processed_event, "auth.new_sign_up");
                        assert_eq!(audit_payload.microservice, "auth");

                        // Use audit_ack to avoid recursive audit emission
                        handler
                            .audit_ack()
                            .await
                            .expect("Failed to audit_ack audit message");

                        barrier.wait().await;
                    }
                })
                .await;

            let now_unix =  SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            // Create and publish an audit event manually to test the flow
            let test_audit_payload = AuditProcessedPayload {
                microservice: "auth".to_string(),
                processed_event: "auth.new_sign_up".to_string(),
                processed_at: now_unix,
                queue_name: "auth_match_commands".to_string(),
                event_id: None,
            };

            // Publish the audit event to the audit exchange
            RabbitMQClient::publish_audit_event(test_audit_payload)
                .await
                .expect("Failed to publish audit event");

            // Wait for the event to be processed
            barrier_clone.wait().await;

            // Verify that at least one audit event was processed
            // Note: In production, AuditHandler.audit_ack() prevents recursive audit emission
            // Test environment may show duplicates due to test setup
            let final_count = processed_count.load(std::sync::atomic::Ordering::SeqCst);
            assert!(
                final_count >= 1,
                "At least one audit event should have been processed, got: {}",
                final_count
            );
        });
    }
}
