use legend_saga::connection::{AvailableMicroservices, RabbitMQClient};
use legend_saga::events::{AuthDeletedUserPayload, MicroserviceEvent};
use legend_saga::events_consume::EventHandler;
use legend_saga::saga::{CommandHandler, StepCommand};
use serde::Deserialize;
use serde_json::json;
use std::error::Error;
use std::time::Duration;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MintPayload {
    image_id: String,
}

async fn handle_mint_image(handler: &CommandHandler) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mint_payload: MintPayload = handler.parse_payload()?;

    println!("Parsed ChangeTemplateId: {mint_payload:?}");

    let json_payload_value = json!({
        "tokenId": "room123",
        "imageId": mint_payload.image_id,
    });
    handler.ack(json_payload_value).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pub const RABBIT_URI: &str = "amqp://rabbit:1234@localhost:5672";

    let r = RabbitMQClient::new(
        RABBIT_URI,
        AvailableMicroservices::TestMint,
        Some(&[
            MicroserviceEvent::TestMint,
            MicroserviceEvent::AuthDeletedUser,
        ]),
    )
    .await?;

    let auth_delete_value = AuthDeletedUserPayload {
        user_id: "user1233".to_string(),
    };

    RabbitMQClient::publish_event(auth_delete_value)
        .await
        .expect("TODO: panic message");

    // r.commence_saga().await.expect("TODO: panic message");

    let s = r.connect_to_saga_commands().await?;
    s.on_with_async_handler(StepCommand::MintImage, |handler| async move {
        match handle_mint_image(&handler).await {
            Ok(_) => {
                println!("Successfully handled MintImage command");
            }
            Err(e) => {
                eprintln!("Error handling MintImage command: {e:?}");
                handler
                    .nack_with_delay(Duration::from_millis(1000), 1)
                    .await
                    .expect("Failed to nack");
            }
        }
    })
    .await;
    let e = r.connect_to_events().await?;

    e.on_with_async_handler(MicroserviceEvent::AuthDeletedUser, |handler| async move {
        async fn handler_fn(handler: &EventHandler) -> Result<(), Box<dyn Error + Send + Sync>> {
            println!(
                "{:?}: {:?}",
                MicroserviceEvent::AuthDeletedUser,
                handler.get_payload()
            );
            let p: AuthDeletedUserPayload = handler.parse_payload()?;
            println!("Payload {p:?} ");
            handler.ack().await?;
            Ok(())
        }

        if let Err(e) = handler_fn(&handler).await {
            eprintln!("Error handling AuthDeletedUser event: {e:?}");
            handler
                .nack_with_delay(Duration::from_millis(1000), 1)
                .await
                .expect("Failed to nack");
        }
    })
    .await;

    println!("Connected to RabbitMQ");
    // block forever

    // Simulate a connection drop while consuming and later publishing an event
    {
        println!("Closing!");
        let conn = r
            .current_connection()
            .await
            .expect("No connection found")
            .write()
            .await;
        conn.close(0, "Test disconnect")
            .await
            .expect("Failed to close connection");
    } // conn dropped here

    // Trigger reconnection
    println!("Reconnection!");
    r.reconnect().await.expect("Reconnection should succeed");
    tokio::signal::ctrl_c().await?;
    Ok(())
}
