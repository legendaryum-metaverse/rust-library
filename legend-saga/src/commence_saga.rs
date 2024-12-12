use crate::queue_consumer_props::Queue;
use lapin::options::QueueDeclareOptions;
use lapin::{options::BasicPublishOptions, types::FieldTable, BasicProperties};
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, EnumIter, EnumString};
use crate::connection::{get_or_init_publish_channel, RabbitMQClient, RabbitMQError};

#[derive(
    Debug, Clone, Copy, AsRefStr, EnumString, PartialEq, EnumIter, Hash, Eq, Deserialize, Serialize,
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum SagaTitle {
    PurchaseResourceFlow,
    RankingsUsersReward,
    TransferCryptoRewardToRankingWinners
}

pub trait PayloadCommenceSaga {
    fn saga_title(&self) -> SagaTitle;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserReward {
    pub user_id: String,
    pub coins: i32,
}
impl PartialEq for UserReward {
    fn eq(&self, other: &Self) -> bool {
        self.user_id == other.user_id && self.coins == other.coins
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RankingsUsersRewardPayload {
    pub rewards: Vec<UserReward>,
}
impl PayloadCommenceSaga for RankingsUsersRewardPayload {
    fn saga_title(&self) -> SagaTitle {
        SagaTitle::RankingsUsersReward
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseResourceFlowPayload {
    pub user_id: String,
    pub resource_id: String,
    pub price: i32,
    pub quantity: i32,
}
impl PayloadCommenceSaga for PurchaseResourceFlowPayload {
    fn saga_title(&self) -> SagaTitle {
        SagaTitle::PurchaseResourceFlow
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CryptoRankingWinners {
    pub user_id: String,
    pub reward: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompletedCryptoRanking {
    pub wallet_address: String,
    pub winners: Vec<CryptoRankingWinners>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransferCryptoRewardToRankingWinnersPayload {
    pub completed_crypto_rankings: Vec<CompletedCryptoRanking>,
}

impl PayloadCommenceSaga for TransferCryptoRewardToRankingWinnersPayload {
    fn saga_title(&self) -> SagaTitle {
        SagaTitle::TransferCryptoRewardToRankingWinners
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct CommenceSaga<T> {
    pub title: SagaTitle,
    pub payload: T, // The payload is a JSON object, Value
}

impl RabbitMQClient {
    pub(crate) async fn send<T: Serialize>(queue_name: &str, payload: &T) -> Result<(), RabbitMQError> {
        let channel_arc = get_or_init_publish_channel().await?;
        let channel = channel_arc.lock().await;

        channel
            .queue_declare(
                queue_name,
                QueueDeclareOptions {
                    durable: true,
                    ..QueueDeclareOptions::default()
                },
                FieldTable::default(),
            )
            .await?;

        let body = serde_json::to_vec(payload)?;

        channel
            .basic_publish(
                "",
                queue_name,
                BasicPublishOptions::default(),
                &body,
                BasicProperties::default()
                    .with_delivery_mode(2) // persistent
                    .with_content_type("application/json".into()),
            )
            .await?;

        Ok(())
    }
    pub async fn commence_saga<T: PayloadCommenceSaga + Serialize>(
        payload: T,
    ) -> Result<(), RabbitMQError> {
        Self::send(
            Queue::COMMENCE_SAGA,
            &CommenceSaga {
                title: payload.saga_title(),
                payload: serde_json::to_value(&payload)?,
            },
        )
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod commence {
    use crate::commence_saga::{
        CommenceSaga, PurchaseResourceFlowPayload, RankingsUsersRewardPayload, SagaTitle,
        UserReward,
    };
    use crate::queue_consumer_props::Queue;
    use crate::test::setup::TestSetup;
    use futures_lite::StreamExt;
    use lapin::options::BasicConsumeOptions;
    use serde_json::json;
    use std::time::Duration;
    use crate::connection::RabbitMQClient;

    /// The saga commence message is sent to the transactional microservice that listens in "commence_saga"
    #[test]
    fn test_commence_saga() {
        let setup = TestSetup::new(None);
        setup.rt.block_on(async {
            let user_id = "user1233";

            let json_payload = json!(
                {
                    "userId": user_id,
                    "resourceId": "resource123",
                    "price": 100,
                    "quantity": 1
                }
            );

            let payload: PurchaseResourceFlowPayload =
                serde_json::from_value(json_payload).unwrap();

            RabbitMQClient::commence_saga(payload).await.unwrap();

            // The transactional microservice receives the message and processes it
            let mut consumer = setup
                .client
                .consume_messages::<CommenceSaga<PurchaseResourceFlowPayload>>(
                    Queue::COMMENCE_SAGA,
                    BasicConsumeOptions::default(),
                )
                .await
                .expect("Failed to create consumer");

            let received_message = tokio::time::timeout(Duration::from_secs(2), consumer.next())
                .await
                .expect("Timed out waiting for message")
                .expect("Failed to receive message")
                .expect("Error in received message");
            assert_eq!(
                received_message.title,
                SagaTitle::PurchaseResourceFlow,
                "Received saga title should match sent title"
            );
            assert_eq!(
                received_message.payload.user_id, user_id,
                "Received message should match sent message"
            );
        });
    }

    #[test]
    fn test_commence_rankings_users_reward_saga() {
        let setup = TestSetup::new(None);
        setup.rt.block_on(async {
            let rewards = vec![
                UserReward {
                    user_id: "user123".to_string(),
                    coins: 100,
                },
                UserReward {
                    user_id: "user456".to_string(),
                    coins: 200,
                },
            ];

            let payload = RankingsUsersRewardPayload {
                rewards: rewards.clone(),
            };

            RabbitMQClient::commence_saga(payload).await.unwrap();

            // The transactional microservice receives the message and processes it
            let mut consumer = setup
                .client
                .consume_messages::<CommenceSaga<RankingsUsersRewardPayload>>(
                    Queue::COMMENCE_SAGA,
                    BasicConsumeOptions::default(),
                )
                .await
                .expect("Failed to create consumer");

            let received_message = tokio::time::timeout(Duration::from_secs(2), consumer.next())
                .await
                .expect("Timed out waiting for message")
                .expect("Failed to receive message")
                .expect("Error in received message");

            assert_eq!(
                received_message.title,
                SagaTitle::RankingsUsersReward,
                "Received saga title should match sent title"
            );
            assert_eq!(
                received_message.payload.rewards, rewards,
                "Received message rewards should match sent rewards"
            );
        });
    }
}
