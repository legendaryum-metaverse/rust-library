use crate::emitter::Emitter;
use crate::events::{EventHandler, MicroserviceEvent};
use crate::queue_consumer_props::{Exchange, QueueConsumerProps};
use crate::saga::{CommandHandler, StepCommand};
use crate::{RabbitMQClient, RabbitMQError};
use tracing::error;

impl RabbitMQClient {
    pub async fn connect_to_events(
        &self,
    ) -> Result<Emitter<EventHandler, MicroserviceEvent>, RabbitMQError> {
        let queue_name = format!("{}_match_commands", self.microservice.as_ref());
        let emitter = Emitter::new();

        self.create_header_consumers(&queue_name, self.events)
            .await?;

        let client = self.clone();
        let emitter_clone = emitter.clone();

        tokio::spawn(async move {
            if let Err(e) = client.consume_events(&queue_name, emitter_clone).await {
                error!("Error consuming messages: {:?}", e);
            }
        });

        Ok(emitter)
    }
    pub async fn connect_to_saga_commands(
        &self,
    ) -> Result<Emitter<CommandHandler, StepCommand>, RabbitMQError> {
        let queue_name = format!("{}_saga_commands", self.microservice.as_ref());
        let emitter = Emitter::new();

        self.create_consumers(vec![QueueConsumerProps {
            queue_name: queue_name.clone(),
            exchange: Exchange::COMMANDS,
        }])
        .await?;

        let client = self.clone();
        let emitter_clone = emitter.clone();

        tokio::spawn(async move {
            if let Err(e) = client.consume_saga_steps(&queue_name, emitter_clone).await {
                error!("Error consuming messages: {:?}", e);
            }
        });

        Ok(emitter)
    }
}
/*
#[cfg(test)]
mod tests {
    use crate::events::{AuthDeletedUserPayload, EventHandler, MicroserviceEvent};
    use crate::saga::{CommandHandler, StepCommand};
    use crate::test::setup::{Config, TestSetup};
    use crate::AvailableMicroservices;
    use serde::Deserialize;
    use serde_json::json;
    use std::error::Error;
    use std::sync::{Arc, Barrier};
    use std::time::Duration;

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct MintPayload {
        image_id: String,
    }

    /// Integration test, TODO: delete or refactor later
    #[test]
    fn test_connect_to_events() {
        let test_events = &[
            MicroserviceEvent::TestImage,
            MicroserviceEvent::AuthDeletedUser,
            MicroserviceEvent::CoinsSendEmail,
        ];
        let setup = TestSetup::new(Some(Config {
            events: test_events,
            microservice: AvailableMicroservices::TestMint,
        }));
        setup.rt.block_on(async {
            // Connect to events
            let e = setup
                .client
                .clone()
                .connect_to_events()
                .await
                .expect("Failed to connect to events");

            let barrier = Arc::new(Barrier::new(3));

            let c_barrier = barrier.clone();

            e.on_with_async_handler(MicroserviceEvent::TestImage, move |handler| async move {
                let res = handler
                    .nack_with_fibonacci_strategy(3, 9)
                    .await
                    .expect("Failed to ack");
                println!("Count, delay, occurrence {:?}", res);
            })
            .await;
            e.on_with_async_handler(MicroserviceEvent::AuthDeletedUser, |handler| async move {
                let res = handler
                    .nack_with_delay(Duration::from_millis(100), 2)
                    .await
                    .expect("Failed to ack");
                println!("Count, delay, occurrence {:?}", res);
            })
            .await;

            e.on_with_async_handler(MicroserviceEvent::CoinsSendEmail, |handler| async move {
                async fn handler_fn(
                    handler: &EventHandler,
                ) -> Result<(), Box<dyn Error + Send + Sync>> {
                    println!(
                        "{:?} {:?}",
                        MicroserviceEvent::AuthDeletedUser,
                        handler.get_payload()
                    );
                    let p: AuthDeletedUserPayload = handler.parse_payload()?;
                    println!("Payload {:?} ", p);
                    handler.ack().await?;
                    Ok(())
                }

                if let Err(e) = handler_fn(&handler).await {
                    eprintln!("Error handling AuthDeletedUser event: {:?}", e);
                    handler
                        .nack_with_delay(Duration::from_millis(1000), 1)
                        .await
                        .expect("Failed to nack");
                }
            })
            .await;

            // Connect to saga
            let s = setup
                .client
                .clone()
                .connect_to_saga_commands()
                .await
                .expect("Failed to connect to events");

            async fn handle_mint_image(
                handler: &CommandHandler,
            ) -> Result<(), Box<dyn Error + Send + Sync>> {
                let mint_payload: MintPayload = handler.parse_payload()?;

                println!("Parsed ChangeTemplateId: {:?}", mint_payload);

                let json_payload_value = json!({
                    "tokenId": "room123",
                    "imageId": mint_payload.image_id,
                });
                handler.ack(json_payload_value).await?;
                Ok(())
            }

            s.on_with_async_handler(StepCommand::MintImage, |handler| async move {
                match handle_mint_image(&handler).await {
                    Ok(_) => {
                        println!("Successfully handled MintImage command");
                    }
                    Err(e) => {
                        eprintln!("Error handling MintImage command: {:?}", e);
                        handler
                            .nack_with_delay(Duration::from_millis(1000), 1)
                            .await
                            .expect("Failed to nack");
                    }
                }
            })
            .await;

            tokio::time::sleep(Duration::from_secs(100000)).await;
        });
    }
}
*/
