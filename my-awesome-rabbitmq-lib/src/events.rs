use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum_macros::{AsRefStr, EnumIter, EnumString};

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
    #[strum(serialize = "legend_rankings.rankings_finished")]
    LegendRankingsRankingsFinished,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TestImagePayload {
    pub image: String,
}

impl PayloadEvent for TestImagePayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::TestImage
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TestMintPayload {
    pub mint: String,
}

impl PayloadEvent for TestMintPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::TestMint
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AuthDeletedUserPayload {
    pub user_id: String,
}

impl PayloadEvent for AuthDeletedUserPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::AuthDeletedUser
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AuthLogoutUserPayload {
    pub user_id: String,
}

impl PayloadEvent for AuthLogoutUserPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::AuthLogoutUser
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LegendMissionsOngoingMissionEventPayload {
    pub redis_key: String,
}

impl PayloadEvent for LegendMissionsOngoingMissionEventPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::LegendMissionsOngoingMission
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub enum RankingsRewardsType {
    Legends,
    CodeExchange,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RankingWinners {
    pub user_id: String,
    pub reward: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompletedRanking {
    pub title: String,
    pub description: String,
    pub author_email: String,
    pub ends_at: String,
    pub reward: String,
    pub reward_type: RankingsRewardsType,
    pub winners: Vec<RankingWinners>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LegendRankingsRankingsFinishedEventPayload {
    pub completed_rankings: Vec<CompletedRanking>,
}

impl PayloadEvent for LegendRankingsRankingsFinishedEventPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::LegendRankingsRankingsFinished
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RoomCreatorCreatedRoomPayload {
    #[serde(rename = "room")]
    pub room: Room,
}

impl PayloadEvent for RoomCreatorCreatedRoomPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::RoomCreatorCreatedRoom
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RoomCreatorUpdatedRoomPayload {
    #[serde(rename = "room")]
    pub room: Room,
}

impl PayloadEvent for RoomCreatorUpdatedRoomPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::RoomCreatorUpdatedRoom
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RoomSnapshotFirstSnapshotPayload {
    pub slug: String,
}

impl PayloadEvent for RoomSnapshotFirstSnapshotPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::RoomSnapshotFirstSnapshot
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SocialNewUserPayload {
    pub user_id: String,
}

impl PayloadEvent for SocialNewUserPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::SocialNewUser
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
