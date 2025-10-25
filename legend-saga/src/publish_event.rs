use crate::events::{AuditPublishedPayload, PayloadEvent};
use crate::queue_consumer_props::Exchange;
use lapin::{
    options::BasicPublishOptions, types::AMQPValue,
    types::FieldTable, BasicProperties,
};
use serde::Serialize;
use crate::connection::{get_or_init_publish_channel, get_stored_microservice, RabbitMQClient, RabbitMQError};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::error;
use uuid::Uuid;

impl RabbitMQClient {
    pub async fn publish_event<T: PayloadEvent + Serialize>(
        payload: T,
    ) -> Result<(), RabbitMQError> {
        let channel_arc = get_or_init_publish_channel().await?;
        let channel = channel_arc.lock().await;

        // Generate UUID v7 for event correlation across all audit events
        let event_id = Uuid::now_v7().to_string();

        // Get publisher microservice name
        let publisher_microservice = get_stored_microservice()?;

        let event_type = payload.event_type();
        let mut header_event = FieldTable::default();
        header_event.insert(
            event_type.as_ref().to_uppercase().into(),
            AMQPValue::LongString(event_type.as_ref().into()),
        );
        header_event.insert("all-micro".into(), AMQPValue::LongString("yes".into()));

        let body = serde_json::to_vec(&payload)?;

        // Publish main event with message properties for tracking
        channel
            .basic_publish(
                Exchange::MATCHING,
                "",
                BasicPublishOptions::default(),
                &body,
                BasicProperties::default()
                    .with_headers(header_event)
                    .with_content_type("application/json".into())
                    .with_delivery_mode(2) // persistent
                    .with_message_id(event_id.clone().into())
                    .with_app_id(publisher_microservice.clone().into()),
            )
            .await?;

        // Emit audit.published event (fire-and-forget - never fail the main flow)
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let audit_payload = AuditPublishedPayload {
            publisher_microservice,
            published_event: event_type.as_ref().to_string(),
            published_at: timestamp,
            event_id,
        };

        // Fire-and-forget: log errors but don't fail the publish operation
        tokio::spawn(async move {
            if let Err(e) = RabbitMQClient::publish_audit_event(audit_payload).await {
                error!("Failed to emit audit.published event: {:?}", e);
            }
        });

        Ok(())
    }

    /// Publishes audit events to the direct audit exchange
    /// Uses the event type as routing key for flexible audit event routing
    pub async fn publish_audit_event<T: PayloadEvent + Serialize>(
        payload: T,
    ) -> Result<(), RabbitMQError> {
        let channel_arc = get_or_init_publish_channel().await?;
        let channel = channel_arc.lock().await;

        // Use the event type as routing key for flexible audit event routing
        let event_type = payload.event_type();
        let routing_key = event_type.as_ref(); // "audit.received", "audit.processed", "audit.dead_letter"

        let body = serde_json::to_vec(&payload)?;

        channel
            .basic_publish(
                Exchange::AUDIT,
                routing_key, // Routes to appropriate queue based on event type
                BasicPublishOptions::default(),
                &body,
                BasicProperties::default()
                    .with_content_type("application/json".into())
                    .with_delivery_mode(2), // persistent
            )
            .await?;

        Ok(())
    }

}

/// Integration test, the micro publishes an event and the "same" microservice client listens to it
#[cfg(test)]
mod test_publish_event {
    use crate::events::AuthDeletedUserPayload;
    use crate::events::MicroserviceEvent::AuthDeletedUser;
    use crate::test::setup::{Config, TestSetup};
    use std::sync::Arc;
    use std::sync::atomic::AtomicUsize;
    use tokio::sync::Barrier;

    use crate::connection::AvailableMicroservices::Auth;
    use crate::connection::RabbitMQClient;

    #[test]
    fn test_publish_event() {
        let setup = TestSetup::new(Some(Config {
            events: &[AuthDeletedUser],
            microservice: Auth,
        }));
        setup.rt.block_on(async {
            let auth_deleted_user_payload = AuthDeletedUserPayload {
                user_id: "user1233".to_string(),
            };

            let barrier = Arc::new(Barrier::new(2));
            let b_clone = barrier.clone();
            let e = setup
                .client
                .connect_to_events()
                .await
                .expect("TODO: panic message");

            e.on_with_async_handler(AuthDeletedUser, move |handler| {
                let barrier = barrier.clone();
                async move {
                    println!("{:?} {:?}", AuthDeletedUser, handler.get_payload());
                    let p: AuthDeletedUserPayload =
                        handler.parse_payload().expect("Error parsing payload");
                    assert_eq!(p.user_id, "user1233");
                    handler.ack().await.expect("Error acking message");
                    barrier.wait().await;
                }
            })
            .await;

            RabbitMQClient::publish_event(auth_deleted_user_payload)
                .await
                .expect("Error publishing AuthDeletedUserPayload event");
            b_clone.wait().await;
        });
    }

    // The emitter reconnection is tested here
    #[test]
    fn test_publish_with_reconnection_event() {
        let setup = TestSetup::new(Some(Config {
            events: &[AuthDeletedUser],
            microservice: Auth,
        }));
        setup.rt.block_on(async {
            let auth_deleted_user_payload = AuthDeletedUserPayload {
                user_id: "user1233".to_string(),
            };

            let e = setup
                .client
                .connect_to_events()
                .await
                .expect("TODO: panic message");

            // The consumption must be last not first, due to reconnection, the "consumer.next().await" will be blocked
            let atomic_counter = Arc::new(AtomicUsize::new(0));

            let first_barrier = Arc::new(Barrier::new(2));
            let first_barrier_clone = first_barrier.clone();

            let last_barrier = Arc::new(Barrier::new(2));
            let last_barrier_clone = last_barrier.clone();

            e.on_with_async_handler(AuthDeletedUser, move |handler| {
                let first_barrier = first_barrier.clone();
                let last_barrier = last_barrier.clone();
                let a_counter = atomic_counter.clone();
                async move {
                    println!("{:?} {:?}", AuthDeletedUser, handler.get_payload());
                    let p: AuthDeletedUserPayload =
                        handler.parse_payload().expect("Error parsing payload");
                    assert_eq!(p.user_id, "user1233");
                    let atomic_value = a_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    handler.ack().await.expect("Error acking message");
                    if atomic_value == 0 {
                        first_barrier.wait().await;
                    }
                    if atomic_value == 1 {
                        last_barrier.wait().await;
                    }
                }
            })
                .await;


            // first publish initiate the publish-channel if not already created
            RabbitMQClient::publish_event(auth_deleted_user_payload.clone())
                .await
                .expect("Error publishing AuthDeletedUserPayload event");

            first_barrier_clone.wait().await;

            // tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

            // Simulate a connection drop while consuming and later publishing an event
            // Step 3: Simulate a connection drop while consuming, topology is erased, so we saved it for later deletion.
            let t = setup.get_current_topology().await;
            {
                let conn = setup.client.current_connection().await.expect("No connection found").write().await;
                conn.close(0, "Test disconnect")
                    .await
                    .expect("Failed to close connection");
            } // conn dropped here

            // Trigger reconnection
            setup
                .client
                .reconnect()
                .await
                .expect("Reconnection should succeed");

            // second publish after reconnection, the channel should be re-established
            RabbitMQClient::publish_event(auth_deleted_user_payload)
                .await
                .expect("Error publishing AuthDeletedUserPayload event");


            last_barrier_clone.wait().await;
            // we must manually delete the before-topology because in "drop" we delete the "after-topology"
            setup.clean_topology(Some(t)).await;
        });
    }
}
