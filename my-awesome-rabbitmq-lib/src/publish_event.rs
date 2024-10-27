use crate::events::PayloadEvent;
use crate::queue_consumer_props::Exchange;
use lapin::{
    options::BasicPublishOptions, options::ExchangeDeclareOptions, types::AMQPValue,
    types::FieldTable, BasicProperties, ExchangeKind,
};
use serde::Serialize;
use crate::connection::{get_or_init_publish_channel, RabbitMQClient, RabbitMQError};

impl RabbitMQClient {
    pub async fn publish_event<T: PayloadEvent + Serialize>(
        payload: T,
    ) -> Result<(), RabbitMQError> {
        let channel_arc = get_or_init_publish_channel().await?;
        let channel = channel_arc.lock().await;

        let event_type = payload.event_type();
        let mut header_event = FieldTable::default();
        header_event.insert(
            event_type.as_ref().to_uppercase().into(),
            AMQPValue::LongString(event_type.as_ref().into()),
        );
        header_event.insert("all-micro".into(), AMQPValue::LongString("yes".into()));

        channel
            .exchange_declare(
                Exchange::MATCHING,
                ExchangeKind::Headers,
                ExchangeDeclareOptions {
                    durable: true,
                    ..ExchangeDeclareOptions::default()
                },
                FieldTable::default(),
            )
            .await?;

        let body = serde_json::to_vec(&payload)?;

        channel
            .basic_publish(
                Exchange::MATCHING,
                "",
                BasicPublishOptions::default(),
                &body,
                BasicProperties::default()
                    .with_headers(header_event)
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

    // #[test]
    // fn test_publish_with_reconnection_event() {
    //     let setup = TestSetup::new(Some(Config {
    //         events: &[AuthDeletedUser],
    //         microservice: Auth,
    //     }));
    //     setup.rt.block_on(async {
    //         let auth_deleted_user_payload = AuthDeletedUserPayload {
    //             user_id: "user1233".to_string(),
    //         };
    //
    //         let e = setup
    //             .client
    //             .connect_to_events()
    //             .await
    //             .expect("TODO: panic message");
    //
    //
    //         // first publish initiated the publishing channel if not already created
    //         RabbitMQClient::publish_event(auth_deleted_user_payload.clone())
    //             .await
    //             .expect("Error publishing AuthDeletedUserPayload event");
    //         tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    //
    //         // Simulate a connection drop while consuming and later publishing an event
    //         {
    //             println!("Closing!");
    //             let conn = setup.client.current_connection().await.expect("No connection found").write().await;
    //             conn.close(0, "Test disconnect")
    //                 .await
    //                 .expect("Failed to close connection");
    //         } // conn dropped here
    //
    //         // Trigger reconnection
    //         println!("Reconnection!");
    //         setup
    //             .client
    //             .reconnect()
    //             .await
    //             .expect("Reconnection should succeed");
    //
    //         println!("Reconnection done");
    //         // second publish after reconnection, the channel should be re-established
    //         RabbitMQClient::publish_event(auth_deleted_user_payload)
    //             .await
    //             .expect("Error publishing AuthDeletedUserPayload event");
    //         println!("LastPublish done");
    //
    //
    //         // The consumption must be last not first, due to reconnection, the "consumer.next().await" will be blocked
    //         //
    //         let atomic_counter = Arc::new(AtomicUsize::new(0));
    //         let barrier = Arc::new(Barrier::new(2));
    //         let b_clone = barrier.clone();
    //         e.on_with_async_handler(AuthDeletedUser, move |handler| {
    //             let barrier = barrier.clone();
    //             let a_counter = atomic_counter.clone();
    //             async move {
    //                 println!("{:?} {:?}", AuthDeletedUser, handler.get_payload());
    //                 let p: AuthDeletedUserPayload =
    //                     handler.parse_payload().expect("Error parsing payload");
    //                 assert_eq!(p.user_id, "user1233");
    //                 handler.ack().await.expect("Error acking message");
    //
    //                 let a_val = a_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    //                 println!("a_val: {}", a_val);
    //                 if a_val == 1{
    //                     barrier.wait().await;
    //                 }
    //             }
    //         })
    //             .await;
    //         b_clone.wait().await;
    //     });
    // }
}
