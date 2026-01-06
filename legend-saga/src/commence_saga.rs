use crate::queue_consumer_props::Queue;
use lapin::{options::BasicPublishOptions, BasicProperties};
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, EnumIter, EnumString};
use crate::connection::{get_or_init_publish_channel, RabbitMQClient, RabbitMQError};

#[derive(
    Debug, Clone, Copy, AsRefStr, EnumString, PartialEq, EnumIter, Hash, Eq, Deserialize, Serialize,
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum SagaTitle {
    TransferCryptoRewardToMissionWinner,
    TransferCryptoRewardToRankingWinners
}

pub trait PayloadCommenceSaga {
    fn saga_title(&self) -> SagaTitle;
}



#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransferCryptoRewardToMissionWinnerPayload {
    /// Wallet address from which rewards will be transferred
    pub wallet_address: String,
    /// ID of the user who completed the mission
    pub user_id: String,
    /// Amount to be transferred
    pub reward: String,
}

impl PayloadCommenceSaga for TransferCryptoRewardToMissionWinnerPayload {
    fn saga_title(&self) -> SagaTitle {
        SagaTitle::TransferCryptoRewardToMissionWinner
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
        CommenceSaga, CompletedCryptoRanking, CryptoRankingWinners,
        SagaTitle, TransferCryptoRewardToMissionWinnerPayload, TransferCryptoRewardToRankingWinnersPayload,
    };
    use crate::queue_consumer_props::Queue;
    use crate::test::setup::TestSetup;
    use futures_lite::StreamExt;
    use lapin::options::{BasicConsumeOptions};
    use std::time::Duration;
    use crate::connection::{RabbitMQClient};

    /// The saga commence message is sent to the transactional microservice that listens in "commence_saga"
    #[test]
    fn test_commence_saga() {
        let setup = TestSetup::new(None);
        setup.rt.block_on(async {
            let user_id = "user1233";
            let wallet_address = "0xabc123";
            let reward = "50";

            // Para que este micro pueda realizar pasos del saga y realizar commence_saga ops las queue's deben existir, no es responsabilidad
            // de los micros crear estos recursos, el micro "transactional" debe crear estos recursos -> "queue.CommenceSaga" en commenceSagaListener
            // y "queue.ReplyToSaga" en startGlobalSagaStepListener

            // The transactional microservice receives the message and processes it
            let mut consumer = setup
                .client
                // Simulando "consumo", "commenceSagaListener" se encuentra implementado en TS y es el responsable de crear la queue.
                .consume_messages::<CommenceSaga<TransferCryptoRewardToMissionWinnerPayload>>(
                    Queue::COMMENCE_SAGA,
                    BasicConsumeOptions::default(),
                )
                .await
                .expect("Failed to create consumer");

            let payload = TransferCryptoRewardToMissionWinnerPayload {
                wallet_address: wallet_address.to_string(),
                user_id: user_id.to_string(),
                reward: reward.to_string(),
            };

            RabbitMQClient::commence_saga(payload).await.unwrap();

            let received_message = tokio::time::timeout(Duration::from_secs(2), consumer.next())
                .await
                .expect("Timed out waiting for message")
                .expect("Failed to receive message")
                .expect("Error in received message");
            assert_eq!(
                received_message.title,
                SagaTitle::TransferCryptoRewardToMissionWinner,
                "Received saga title should match sent title"
            );
            assert_eq!(
                received_message.payload.user_id, user_id,
                "Received message should match sent message"
            );
        });
    }

    #[test]
    fn test_commence_transfer_crypto_reward_to_ranking_winners_saga() {
        let setup = TestSetup::new(None);
        setup.rt.block_on(async {
            let completed_crypto_rankings = vec![CompletedCryptoRanking {
                wallet_address: "0x1234567890abcdef".to_string(),
                winners: vec![
                    CryptoRankingWinners {
                        user_id: "user123".to_string(),
                        reward: "100".to_string(),
                    },
                    CryptoRankingWinners {
                        user_id: "user456".to_string(),
                        reward: "200".to_string(),
                    },
                ],
            }];

            // Para que este micro pueda realizar pasos del saga y realizar commence_saga ops las queue's deben existir, no es responsabilidad
            // de los micros crear estos recursos, el micro "transactional" debe crear estos recursos -> "queue.CommenceSaga" en commenceSagaListener
            // y "queue.ReplyToSaga" en startGlobalSagaStepListener

            // The transactional microservice receives the message and processes it
            let mut consumer = setup
                .client
                // Simulando "consumo", "commenceSagaListener" se encuentra implementado en TS y es el responsable de crear la queue.
                .consume_messages::<CommenceSaga<TransferCryptoRewardToRankingWinnersPayload>>(
                    Queue::COMMENCE_SAGA,
                    BasicConsumeOptions::default(),
                )
                .await
                .expect("Failed to create consumer");

            let payload = TransferCryptoRewardToRankingWinnersPayload {
                completed_crypto_rankings: completed_crypto_rankings.clone(),
            };
            RabbitMQClient::commence_saga(payload).await.unwrap();

            let received_message = tokio::time::timeout(Duration::from_secs(2), consumer.next())
                .await
                .expect("Timed out waiting for message")
                .expect("Failed to receive message")
                .expect("Error in received message");

            assert_eq!(
                received_message.title,
                SagaTitle::TransferCryptoRewardToRankingWinners,
                "Received saga title should match sent title"
            );
            assert_eq!(
                received_message.payload.completed_crypto_rankings.len(),
                completed_crypto_rankings.len(),
                "Received message should have same number of rankings"
            );
            assert_eq!(
                received_message.payload.completed_crypto_rankings[0].wallet_address,
                completed_crypto_rankings[0].wallet_address,
                "Wallet address should match"
            );
            assert_eq!(
                received_message.payload.completed_crypto_rankings[0].winners.len(),
                completed_crypto_rankings[0].winners.len(),
                "Winners count should match"
            );
        });
    }
}
