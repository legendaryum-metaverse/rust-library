use crate::emitter::Emitter;
use crate::events::{ MicroserviceEvent};
use crate::queue_consumer_props::{Exchange, QueueConsumerProps};
use crate::saga::{CommandHandler, StepCommand};
use tracing::error;
use crate::connection::{RabbitMQClient, RabbitMQError};
use crate::events_consume::EventHandler;
pub(crate) type EventEmitter = Emitter<EventHandler, MicroserviceEvent>;
pub(crate) type SagaEmitter = Emitter<CommandHandler, StepCommand>;

impl RabbitMQClient {
    pub async fn connect_to_events(
        &self,
    ) -> Result<EventEmitter, RabbitMQError> {
        let queue_name = self.events_queue_name.clone();
        self.create_header_consumers(&queue_name, self.events)
            .await?;
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
}
