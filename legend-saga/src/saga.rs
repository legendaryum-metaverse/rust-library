use crate::emitter::Emitter;
use crate::my_delivery::MyDelivery;
use crate::nack::Nack;
use crate::queue_consumer_props::Queue;
use futures_lite::StreamExt;
use lapin::options::{
    BasicAckOptions, BasicConsumeOptions, BasicNackOptions, BasicQosOptions,
};
use lapin::types::FieldTable;
use lapin::Channel;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};
use tracing::error;
use crate::connection::{AvailableMicroservices, RabbitMQClient, RabbitMQError};

#[derive(
    Debug, Clone, PartialEq, Eq, EnumString, AsRefStr, EnumIter, Serialize, Deserialize, Hash,
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum StepCommand {
    // TestImage commands
    CreateImage,
    UpdateToken,

    // TestMint commands
    MintImage,

    // Auth commands
    CreateUser,

    // Coins commands
    #[strum(serialize = "resource_purchased:deduct_coins")]
    #[serde(rename = "resource_purchased:deduct_coins")]
    ResourcePurchasedDeductCoins,
    #[strum(serialize = "rankings_users_reward:reward_coins")]
    #[serde(rename = "rankings_users_reward:reward_coins")]
    RankingsRewardCoins,

    // RoomInventory commands
    #[strum(serialize = "resource_purchased:save_purchased_resource")]
    #[serde(rename = "resource_purchased:save_purchased_resource")]
    ResourcePurchasedSavePurchasedResource,

    // RoomCreator commands
    UpdateIslandRoomTemplate,

    // Showcase commands
    RandomizeIslandPvImage,

    // Social commands
    #[strum(serialize = "update_user:image")]
    #[serde(rename = "update_user:image")]
    UpdateUserImage,
    CreateSocialUser,

    // Storage commands
    UploadFile,
}

#[derive(
    Debug, Serialize, Deserialize, PartialEq, Eq, EnumString, Display, AsRefStr, EnumIter, Clone,
)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
enum Status {
    Success,
    Failure,
    Sent,
    Pending,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct SagaStep {
    microservice: AvailableMicroservices, // Assuming this type exists
    command: StepCommand,
    status: Status,
    saga_id: i32,
    payload: HashMap<String, Value>,
    previous_payload: HashMap<String, Value>,
    is_current_step: bool,
}

#[derive(Clone)]
pub struct CommandHandler {
    channel: MicroserviceConsumeChannel,
    payload: HashMap<String, Value>,
    #[allow(dead_code)]
    saga_id: i32,
}

impl CommandHandler {
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

    pub async fn ack(&self, payload_for_next_step: Value) -> Result<(), RabbitMQError> {
        self.channel.ack(payload_for_next_step).await
    }

    pub async fn nack_with_delay(
        &self,
        delay: Duration,
        max_retries: i32,
    ) -> Result<(i32, Duration), RabbitMQError> {
        self.channel.nack.with_delay(delay, max_retries).await
    }

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
struct MicroserviceConsumeChannel {
    channel: Channel,
    delivery: MyDelivery,
    #[allow(dead_code)]
    queue_name: String,
    step: SagaStep,
    nack: Nack,
}

impl RabbitMQClient {
    pub(crate) async fn consume_saga_steps(
        &self,
        queue_name: &str,
        emitter: Emitter<CommandHandler, StepCommand>,
    ) -> Result<(), RabbitMQError> {
        let channel = self.saga_channel.lock().await;
        channel.basic_qos(1, BasicQosOptions::default()).await?;

        let mut consumer = channel
            .basic_consume(
                queue_name,
                "saga_consumer",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;

        // it needs to drop manually, next is an infinite loop
        drop(channel);

        while let Some(delivery) = consumer.next().await {
            match delivery {
                Ok(delivery) => {
                    if let Err(e) = self.handle_saga_step(&delivery, &emitter, queue_name).await {
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

    async fn handle_saga_step(
        &self,
        delivery: &lapin::message::Delivery,
        emitter: &Emitter<CommandHandler, StepCommand>,
        queue_name: &str,
    ) -> Result<(), RabbitMQError> {
        let current_step: SagaStep = serde_json::from_slice(&delivery.data)?;
        let channel = self.saga_channel.lock().await;
        let delivery = MyDelivery::new(delivery);

        let command = current_step.command.clone();
        let saga_id = current_step.saga_id;
        let previous_payload = current_step.previous_payload.clone();

        let response_channel = MicroserviceConsumeChannel::new(
            channel.clone(),
            delivery,
            queue_name.to_string(),
            current_step,
        );

        let event_handler = CommandHandler {
            payload: previous_payload,
            channel: response_channel,
            saga_id,
        };

        emitter.emit(command, event_handler).await;
        Ok(())
    }
}

impl MicroserviceConsumeChannel {
    fn new(channel: Channel, delivery: MyDelivery, queue_name: String, step: SagaStep) -> Self {
        let nack = Nack::new(channel.clone(), delivery.clone(), queue_name.clone());
        Self {
            channel,
            delivery,
            queue_name,
            step,
            nack,
        }
    }
    async fn ack(&self, payload_for_next_step: Value) -> Result<(), RabbitMQError> {
        let mut step = self.step.clone();
        step.status = Status::Success;

        let mut next_payload = HashMap::new();

        // Transfer metadata from previous payload
        for (key, value) in step.previous_payload.iter() {
            if key.len() > 2 && key.starts_with("__") {
                next_payload.insert(key.clone(), value.clone());
            }
        }

        //  Alternative if  payload_for_next_step: HashMap<String, Value>, u can iter, but it misses object check
        //   for (key, value) in payload_for_next_step.iter() {
        //             next_payload.insert(key.clone(), value.clone());
        //         }
        match payload_for_next_step {
            Value::Object(map) => {
                for (key, value) in map {
                    next_payload.insert(key, value);
                }
            }
            _ => {
                return Err(RabbitMQError::InvalidPayload(
                    "Expected an object".to_string(),
                ))
            }
        }

        step.payload = next_payload;

        RabbitMQClient::send(Queue::REPLY_TO_SAGA, &step).await?;

        self.channel
            .basic_ack(self.delivery.delivery_tag, BasicAckOptions::default())
            .await
            .map_err(RabbitMQError::from)
    }
}
