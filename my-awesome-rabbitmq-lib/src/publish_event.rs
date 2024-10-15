use crate::events::PayloadEvent;
use crate::queue_consumer_props::Exchange;
use crate::{RabbitMQClient, RabbitMQError};
use lapin::{
    options::BasicPublishOptions, options::ExchangeDeclareOptions, types::AMQPValue,
    types::FieldTable, BasicProperties, ExchangeKind,
};
use serde::Serialize;

impl RabbitMQClient {
    pub async fn publish_event<T: PayloadEvent + Serialize>(
        &self,
        payload: T,
    ) -> Result<(), RabbitMQError> {
        let channel = self.events_channel.lock().await;

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
    use crate::AvailableMicroservices::Auth;
    use std::sync::Arc;
    use tokio::sync::Barrier;

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

            setup
                .client
                .publish_event(auth_deleted_user_payload)
                .await
                .expect("Error publishing AuthDeletedUserPayload event");
            b_clone.wait().await;
        });
    }
}
