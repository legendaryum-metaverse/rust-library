use my_awesome_rabbitmq_lib::connection::{AvailableMicroservices, RabbitMQClient};
use my_awesome_rabbitmq_lib::events::{AuthDeletedUserPayload, MicroserviceEvent};
use std::error::Error;
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pub const RABBIT_URI: &str = "amqp://rabbit:1234@localhost:5672";

    RabbitMQClient::new(
        RABBIT_URI,
        AvailableMicroservices::TestMint,
        Some(&[MicroserviceEvent::AuthDeletedUser]),
    )
    .await?;

    let random_number_from_bytes = {
        use std::time::{SystemTime, UNIX_EPOCH};
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        since_the_epoch.as_secs() ^ since_the_epoch.subsec_nanos() as u64
    };

    let auth_delete_value = AuthDeletedUserPayload {
        user_id: random_number_from_bytes.to_string(),
    };

    RabbitMQClient::publish_event(auth_delete_value)
        .await
        .expect("TODO: panic message");

    Ok(())
}
