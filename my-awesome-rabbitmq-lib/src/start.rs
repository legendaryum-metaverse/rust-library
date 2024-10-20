use crate::emitter::Emitter;
use crate::events::{ MicroserviceEvent};
use crate::queue_consumer_props::{Exchange, QueueConsumerProps};
use crate::saga::{CommandHandler, StepCommand};
use tracing::error;
use crate::connection::{RabbitMQClient, RabbitMQError};
use crate::events_consume::EventHandler;

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
