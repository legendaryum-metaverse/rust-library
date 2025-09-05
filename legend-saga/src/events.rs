use chrono::{DateTime, Utc};
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
    #[strum(serialize = "legend_missions.new_mission_created")]
    LegendMissionsNewMissionCreated,
    #[strum(serialize = "legend_missions.ongoing_mission")]
    LegendMissionsOngoingMission,
    #[strum(serialize = "legend_missions.mission_finished")]
    LegendMissionsMissionFinished,
    #[strum(serialize = "legend_missions.send_email_crypto_mission_completed")]
    LegendMissionsSendEmailCryptoMissionCompleted,
    #[strum(serialize = "legend_missions.send_email_code_exchange_mission_completed")]
    LegendMissionsSendEmailCodeExchangeMissionCompleted,
    #[strum(serialize = "legend_missions.send_email_nft_mission_completed")]
    LegendMissionsSendEmailNftMissionCompleted,
    #[strum(serialize = "legend_rankings.rankings_finished")]
    LegendRankingsRankingsFinished,
    #[strum(serialize = "legend_showcase.product_virtual_deleted")]
    LegendShowcaseProductVirtualDeleted,
    #[strum(serialize = "legend_showcase.update_allowed_mission_subscription_ids")]
    LegendShowcaseUpdateAllowedMissionSubscriptionIds,
    #[strum(serialize = "legend_showcase.update_allowed_ranking_subscription_ids")]
    LegendShowcaseUpdateAllowedRankingSubscriptionIds,
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
    #[strum(serialize = "social.updated_user")]
    SocialUpdatedUser,
    #[strum(serialize = "social_media_rooms.delete_in_batch")]
    SocialMediaRoomsDeleteInBatch,
    #[strum(serialize = "legend_rankings.new_ranking_created")]
    LegendRankingsNewRankingCreated,
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

/// Represents the fields that will be sent by email when a mission is created.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LegendMissionsNewMissionCreatedEventPayload {
    pub title: String,
    pub author: String,
    pub author_email: String,
    pub reward: i32,
    pub start_date: String,
    pub end_date: String,
    pub max_players_claiming_reward: i32,
    pub time_to_reward: i32,
    pub notification_config: Option<NotificationConfig>,
}

impl PayloadEvent for LegendMissionsNewMissionCreatedEventPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::LegendMissionsNewMissionCreated
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MissionFinishedParticipant {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LegendMissionsMissionFinishedEventPayload {
    pub mission_title: String,
    pub participants: Vec<MissionFinishedParticipant>,
}

impl PayloadEvent for LegendMissionsMissionFinishedEventPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::LegendMissionsMissionFinished
    }
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
    pub reward_type: String,
    pub winners: Vec<RankingWinners>,
    // Present only if reward_type is "Nft"
    pub nft_blockchain_network: Option<String>,
    pub nft_contract_address: Option<String>,
    // Present only if reward_type is "Crypto"
    pub wallet_crypto_asset: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LegendMissionsSendEmailCryptoMissionCompletedPayload {
    pub user_id: String,
    pub mission_title: String,
    pub reward: String,
    pub blockchain_network: String,
    pub crypto_asset: String,
}

impl PayloadEvent for LegendMissionsSendEmailCryptoMissionCompletedPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::LegendMissionsSendEmailCryptoMissionCompleted
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LegendMissionsSendEmailCodeExchangeMissionCompletedPayload {
    pub user_id: String,
    pub mission_title: String,
    pub code_value: String,
    pub code_description: String,
}

impl PayloadEvent for LegendMissionsSendEmailCodeExchangeMissionCompletedPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::LegendMissionsSendEmailCodeExchangeMissionCompleted
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LegendMissionsSendEmailNftMissionCompletedPayload {
    pub user_id: String,
    pub mission_title: String,
    pub nft_contract_address: String,
    pub nft_token_id: String,
}

impl PayloadEvent for LegendMissionsSendEmailNftMissionCompletedPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::LegendMissionsSendEmailNftMissionCompleted
    }
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
pub struct LegendShowcaseProductVirtualDeletedEventPayload {
    /// Unique identifier of the deleted virtual product
    pub product_virtual_id: String,
    /// Slug of the deleted virtual product
    pub product_virtual_slug: String,
}

impl PayloadEvent for LegendShowcaseProductVirtualDeletedEventPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::LegendShowcaseProductVirtualDeleted
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LegendShowcaseUpdateAllowedMissionSubscriptionIdsEventPayload {
    pub product_virtual_slug: String,
    pub allowed_subscription_ids: Vec<String>,
}

impl PayloadEvent for LegendShowcaseUpdateAllowedMissionSubscriptionIdsEventPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::LegendShowcaseUpdateAllowedMissionSubscriptionIds
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LegendShowcaseUpdateAllowedRankingSubscriptionIdsEventPayload {
    pub product_virtual_id: String,
    pub allowed_subscription_ids: Vec<String>,
}

impl PayloadEvent for LegendShowcaseUpdateAllowedRankingSubscriptionIdsEventPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::LegendShowcaseUpdateAllowedRankingSubscriptionIds
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

/// Gender represents the possible genders a social user can have.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum Gender {
    Male,
    Female,
    Undefined,
}

/// Represents the geographical location of a user
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserLocation {
    pub continent: String,
    pub country: String,
    pub region: String,
    pub city: String,
}

/// SocialUser represents the social user model.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SocialUser {
    #[serde(rename = "_id")]
    pub id: String,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    pub gender: Gender,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_public_profile: Option<bool>,
    pub followers: Vec<String>,
    pub following: Vec<String>,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birthday: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<UserLocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_screenshot: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub glb_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub social_media: Option<HashMap<String, String>>,
    pub preferences: Vec<String>,
    pub blocked_users: Vec<String>,
    #[serde(rename = "RPMAvatarId", skip_serializing_if = "Option::is_none")]
    pub rpm_avatar_id: Option<String>,
    #[serde(rename = "RPMUserId", skip_serializing_if = "Option::is_none")]
    pub rpm_user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paid_price_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SocialNewUserPayload {
    pub social_user: SocialUser,
}

impl PayloadEvent for SocialNewUserPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::SocialNewUser
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SocialUpdatedUserPayload {
    pub social_user: SocialUser,
}

impl PayloadEvent for SocialUpdatedUserPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::SocialUpdatedUser
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NotificationConfig {
    pub custom_emails: Option<Vec<String>>,
    pub template_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LegendRankingsNewRankingCreatedEventPayload {
    pub title: String,
    pub description: String,
    pub author_email: String,
    pub reward_type: String,
    pub ends_at: String,
    pub nft_blockchain_network: Option<String>,
    pub nft_contract_address: Option<String>,
    pub wallet_crypto_asset: Option<String>,
    pub notification_config: Option<NotificationConfig>,
}

impl PayloadEvent for LegendRankingsNewRankingCreatedEventPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::LegendRankingsNewRankingCreated
    }
}
