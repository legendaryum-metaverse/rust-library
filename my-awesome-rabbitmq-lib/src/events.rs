use crate::emitter::Emitter;
use crate::my_delivery::MyDelivery;
use crate::nack::Nack;
use crate::{RabbitMQClient, RabbitMQError};
use futures_lite::StreamExt;
use lapin::options::{BasicAckOptions, BasicConsumeOptions, BasicNackOptions, BasicQosOptions};
use lapin::types::{AMQPValue, FieldTable};
use lapin::Channel;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::time::Duration;
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, EnumIter, EnumString};
use tracing::{error, info};

/// Represents the available events in the system.
#[derive(Debug, Clone, Copy, AsRefStr, EnumString, PartialEq, EnumIter, Hash, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum MicroserviceEvent {
    #[strum(serialize = "test.image")]
    TestImage,
    #[strum(serialize = "test.mint")]
    TestMint,
    #[strum(serialize = "auth.deleted_user")]
    AuthDeletedUser,
    #[strum(serialize = "auth.logout_user")]
    AuthLogoutUser,
    #[strum(serialize = "auth.new_user")]
    AuthNewUser,
    #[strum(serialize = "coins.notify_client")]
    CoinsNotifyClient,
    #[strum(serialize = "coins.send_email")]
    CoinsSendEmail,
    #[strum(serialize = "coins.update_subscription")]
    CoinsUpdateSubscription,
    #[strum(serialize = "legend_missions.completed_mission_reward")]
    LegendMissionsCompletedMissionReward,
    #[strum(serialize = "legend_missions.ongoing_mission")]
    LegendMissionsOngoingMission,
    #[strum(serialize = "room_creator.created_room")]
    RoomCreatorCreatedRoom,
    #[strum(serialize = "room_creator.updated_room")]
    RoomCreatorUpdatedRoom,
    #[strum(serialize = "room_inventory.update_vp_building_image")]
    RoomInventoryUpdateVpBuildingImage,
    #[strum(serialize = "room_snapshot.building_change_in_island")]
    RoomSnapshotBuildingChangeInIsland,
    #[strum(serialize = "room_snapshot.first_snapshot")]
    RoomSnapshotFirstSnapshot,
    #[strum(serialize = "social.block_chat")]
    SocialBlockChat,
    #[strum(serialize = "social.new_user")]
    SocialNewUser,
    #[strum(serialize = "social.unblock_chat")]
    SocialUnblockChat,
    #[strum(serialize = "social_media_rooms.delete_in_batch")]
    SocialMediaRoomsDeleteInBatch,
}

pub trait PayloadEvent {
    fn event_type(&self) -> MicroserviceEvent;
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestImagePayload {
    pub image: String,
}

impl PayloadEvent for TestImagePayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::TestImage
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestMintPayload {
    pub mint: String,
}

impl PayloadEvent for TestMintPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::TestMint
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthDeletedUserPayload {
    pub user_id: String,
}

impl PayloadEvent for AuthDeletedUserPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::AuthDeletedUser
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthLogoutUserPayload {
    pub user_id: String,
}

impl PayloadEvent for AuthLogoutUserPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::AuthLogoutUser
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthNewUserPayload {
    pub id: String,
    pub email: String,
    pub username: String,
    pub userlastname: String,
}

impl PayloadEvent for AuthNewUserPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::AuthNewUser
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoinsUpdateSubscriptionPayload {
    pub user_id: String,
    pub paid_price_id: String,
}

impl PayloadEvent for CoinsUpdateSubscriptionPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::CoinsUpdateSubscription
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegendMissionsCompletedMissionRewardEventPayload {
    pub user_id: String,
    pub coins: i32,
}

impl PayloadEvent for LegendMissionsCompletedMissionRewardEventPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::LegendMissionsCompletedMissionReward
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegendMissionsOngoingMissionEventPayload {
    pub redis_key: String,
}

impl PayloadEvent for LegendMissionsOngoingMissionEventPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::LegendMissionsOngoingMission
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoinsNotifyClientPayload {
    pub room: String,
    pub message: HashMap<String, serde_json::Value>,
}

impl PayloadEvent for CoinsNotifyClientPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::CoinsNotifyClient
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoinsSendEmailPayload {
    pub user_id: String,
    pub email_type: String,
    pub email: String,
    pub coins: i32,
}

impl PayloadEvent for CoinsSendEmailPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::CoinsSendEmail
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Room {
    #[serde(rename = "Id")]
    pub id: String,
    #[serde(rename = "CreateAt")]
    pub create_at: String,
    #[serde(rename = "UpdateAt")]
    pub update_at: String,
    #[serde(rename = "type")]
    pub room_type: String,
    pub name: String,
    pub owner_id: String,
    pub owner_email: String,
    pub max_players: i32,
    pub max_layers: i32,
    pub template_id: String,
    pub have_editor: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoomCreatorCreatedRoomPayload {
    #[serde(rename = "room")]
    pub room: Room,
}

impl PayloadEvent for RoomCreatorCreatedRoomPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::RoomCreatorCreatedRoom
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoomCreatorUpdatedRoomPayload {
    #[serde(rename = "room")]
    pub room: Room,
}

impl PayloadEvent for RoomCreatorUpdatedRoomPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::RoomCreatorUpdatedRoom
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomInventoryUpdateVpBuildingImagePayload {
    pub images: Vec<String>,
    pub room_type: String,
    pub user_id: String,
}

impl PayloadEvent for RoomInventoryUpdateVpBuildingImagePayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::RoomInventoryUpdateVpBuildingImage
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomSnapshotBuildingChangeInIslandPayload {
    pub building: String,
    pub user_id: String,
}

impl PayloadEvent for RoomSnapshotBuildingChangeInIslandPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::RoomSnapshotBuildingChangeInIsland
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomSnapshotFirstSnapshotPayload {
    pub slug: String,
}

impl PayloadEvent for RoomSnapshotFirstSnapshotPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::RoomSnapshotFirstSnapshot
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SocialBlockChatPayload {
    pub user_id: String,
    pub user_to_block_id: String,
}

impl PayloadEvent for SocialBlockChatPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::SocialBlockChat
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SocialMediaRoomsDeleteInBatchPayload {
    pub bucket_name: String,
    pub file_paths: Vec<String>,
}

impl PayloadEvent for SocialMediaRoomsDeleteInBatchPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::SocialMediaRoomsDeleteInBatch
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SocialNewUserPayload {
    pub user_id: String,
}

impl PayloadEvent for SocialNewUserPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::SocialNewUser
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SocialUnblockChatPayload {
    pub user_id: String,
    pub user_to_unblock_id: String,
}

impl PayloadEvent for SocialUnblockChatPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::SocialUnblockChat
    }
}

/// # EventHandler

#[derive(Clone)]
pub struct EventHandler {
    payload: HashMap<String, Value>,
    channel: EventsConsumeChannel,
}
impl EventHandler {
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

    pub async fn ack(&self) -> Result<(), RabbitMQError> {
        self.channel.ack().await
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

impl RabbitMQClient {
    pub(crate) async fn consume_events(
        &self,
        queue_name: &str,
        emitter: Emitter<EventHandler, MicroserviceEvent>,
    ) -> Result<(), RabbitMQError> {
        let channel = self.events_channel.lock().await;
        channel.basic_qos(1, BasicQosOptions::default()).await?;

        let mut consumer = channel
            .basic_consume(
                queue_name,
                "event_consumer",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;

        // it needs to drop manually, next is an infinite loop
        drop(channel);

        while let Some(delivery) = consumer.next().await {
            match delivery {
                Ok(delivery) => {
                    if let Err(e) = self.handle_event(&delivery, &emitter, queue_name).await {
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

    async fn handle_event(
        &self,
        delivery: &lapin::message::Delivery,
        emitter: &Emitter<EventHandler, MicroserviceEvent>,
        queue_name: &str,
    ) -> Result<(), RabbitMQError> {
        let payload: HashMap<String, Value> = serde_json::from_slice(&delivery.data)?;

        let event_key =
            Self::find_event_values(&delivery.properties.headers().clone().unwrap_or_default())?;

        if event_key.len() > 1 {
            info!("More than one valid header, using the first one detected");
        }

        let event = &event_key[0];

        let channel = self.events_channel.lock().await;
        let delivery = MyDelivery::new(delivery);

        let response_channel =
            EventsConsumeChannel::new(channel.clone(), delivery, queue_name.to_string());

        let event_handler = EventHandler {
            payload,
            channel: response_channel,
        };

        emitter.emit(*event, event_handler).await;

        Ok(())
    }

    fn find_event_values(headers: &FieldTable) -> Result<Vec<MicroserviceEvent>, RabbitMQError> {
        let valid_events: HashSet<_> = MicroserviceEvent::iter().collect();

        let event_values: Vec<MicroserviceEvent> = headers
            .inner()
            .iter()
            .filter_map(|(_, value)| {
                if let AMQPValue::LongString(s) = value {
                    let event_str = s.to_string();
                    MicroserviceEvent::from_str(&event_str).ok()
                } else {
                    None
                }
            })
            .filter(|event| valid_events.contains(event))
            .collect();

        if event_values.is_empty() {
            Err(RabbitMQError::InvalidHeader)
        } else {
            Ok(event_values)
        }
    }
}

#[derive(Clone)]
struct EventsConsumeChannel {
    channel: Channel,
    delivery: MyDelivery,
    #[allow(dead_code)]
    queue_name: String,
    nack: Nack,
}

impl EventsConsumeChannel {
    fn new(channel: Channel, delivery: MyDelivery, queue_name: String) -> Self {
        Self {
            channel: channel.clone(),
            delivery: delivery.clone(),
            queue_name: queue_name.clone(),
            nack: Nack::new(channel, delivery, queue_name),
        }
    }

    async fn ack(&self) -> Result<(), RabbitMQError> {
        self.channel
            .basic_ack(self.delivery.delivery_tag, BasicAckOptions::default())
            .await
            .map_err(RabbitMQError::from)
    }
}

#[cfg(test)]
mod test_events {
    use super::*;
    use lapin::types::ShortString;

    #[test]
    fn test_find_event_values() {
        let mut headers = FieldTable::default();
        headers.insert(
            ShortString::from("event1"),
            AMQPValue::LongString("auth.deleted_user".into()),
        );
        headers.insert(
            ShortString::from("event2"),
            AMQPValue::LongString("invalid_event".into()),
        );
        headers.insert(
            ShortString::from("event3"),
            AMQPValue::LongString("social.new_user".into()),
        );

        let result = RabbitMQClient::find_event_values(&headers);
        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 2);
        assert!(events.contains(&MicroserviceEvent::AuthDeletedUser));
        assert!(events.contains(&MicroserviceEvent::SocialNewUser));
    }

    #[test]
    fn test_find_event_values_no_valid_events() {
        let mut headers = FieldTable::default();
        headers.insert(
            ShortString::from("event1"),
            AMQPValue::LongString("invalid_event1".into()),
        );
        headers.insert(
            ShortString::from("event2"),
            AMQPValue::LongString("invalid_event2".into()),
        );

        let result = RabbitMQClient::find_event_values(&headers);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RabbitMQError::InvalidHeader));
    }
}
