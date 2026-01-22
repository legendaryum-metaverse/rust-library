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
    /// Emitted when an event is received by a microservice before processing starts (audit tracking)
    #[strum(serialize = "audit.received")]
    AuditReceived,
    /// Emitted when an event is successfully processed by a microservice for audit tracking
    #[strum(serialize = "audit.processed")]
    AuditProcessed,
    /// Emitted when a message is rejected/nacked and sent to dead letter queue
    #[strum(serialize = "audit.dead_letter")]
    AuditDeadLetter,
    /// Emitted when an event is published by a microservice (audit tracking)
    #[strum(serialize = "audit.published")]
    AuditPublished,
    #[strum(serialize = "auth.deleted_user")]
    AuthDeletedUser,
    #[strum(serialize = "auth.logout_user")]
    AuthLogoutUser,
    #[strum(serialize = "auth.new_user")]
    AuthNewUser,
    #[strum(serialize = "auth.blocked_user")]
    AuthBlockedUser,
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
    #[strum(serialize = "social.block_chat")]
    SocialBlockChat,
    #[strum(serialize = "social.new_user")]
    SocialNewUser,
    #[strum(serialize = "social.unblock_chat")]
    SocialUnblockChat,
    #[strum(serialize = "social.updated_user")]
    SocialUpdatedUser,
    #[strum(serialize = "legend_rankings.new_ranking_created")]
    LegendRankingsNewRankingCreated,
    #[strum(serialize = "legend_rankings.intermediate_reward")]
    LegendRankingsIntermediateReward,
    #[strum(serialize = "legend_rankings.participation_reward")]
    LegendRankingsParticipationReward,
    // Billing events - Payment and subscription domain events (No Stripe leakage)
    #[strum(serialize = "billing.payment_created")]
    BillingPaymentCreated,
    #[strum(serialize = "billing.payment_succeeded")]
    BillingPaymentSucceeded,
    #[strum(serialize = "billing.payment_failed")]
    BillingPaymentFailed,
    #[strum(serialize = "billing.payment_refunded")]
    BillingPaymentRefunded,
    #[strum(serialize = "billing.subscription_created")]
    BillingSubscriptionCreated,
    #[strum(serialize = "billing.subscription_updated")]
    BillingSubscriptionUpdated,
    #[strum(serialize = "billing.subscription_renewed")]
    BillingSubscriptionRenewed,
    #[strum(serialize = "billing.subscription_canceled")]
    BillingSubscriptionCanceled,
    #[strum(serialize = "billing.subscription_expired")]
    BillingSubscriptionExpired,
    // Legend Events - Event and registration domain events
    #[strum(serialize = "legend_events.new_event_created")]
    LegendEventsNewEventCreated,
    #[strum(serialize = "legend_events.event_started")]
    LegendEventsEventStarted,
    #[strum(serialize = "legend_events.event_ended")]
    LegendEventsEventEnded,
    #[strum(serialize = "legend_events.player_registered")]
    LegendEventsPlayerRegistered,
    #[strum(serialize = "legend_events.player_joined_waitlist")]
    LegendEventsPlayerJoinedWaitlist,
    #[strum(serialize = "legend_events.score_submitted")]
    LegendEventsScoreSubmitted,
    #[strum(serialize = "legend_events.events_finished")]
    LegendEventsEventsFinished,
    #[strum(serialize = "legend_events.intermediate_reward")]
    LegendEventsIntermediateReward,
    #[strum(serialize = "legend_events.participation_reward")]
    LegendEventsParticipationReward,
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
pub struct AuthBlockedUserPayload {
    pub user_id: String,
    pub block_type: String,
    pub block_reason: Option<String>,
    pub block_expiration_hours: Option<i32>,
}

impl PayloadEvent for AuthBlockedUserPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::AuthBlockedUser
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
    pub reward: String,
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
    /// Optional notification config (JSON) to enrich email templates
    pub notification_config: Option<serde_json::Value>,
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
pub struct SocialBlockChatPayload {
    pub user_id: String,
    pub user_to_block_id: String,
}

impl PayloadEvent for SocialBlockChatPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::SocialBlockChat
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
    pub start_at: String,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LegendRankingsIntermediateRewardEventPayload {
    pub user_id: String,
    pub ranking_id: i32,
    pub intermediate_reward_type: String,
    pub reward_config: serde_json::Value,
    pub template_name: String,
    pub template_data: serde_json::Value,
}

impl PayloadEvent for LegendRankingsIntermediateRewardEventPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::LegendRankingsIntermediateReward
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LegendRankingsParticipationRewardEventPayload {
    pub user_id: String,
    pub ranking_id: i32,
    pub participation_reward_type: String,
    pub reward_config: serde_json::Value,
    pub template_name: String,
    pub template_data: serde_json::Value,
}

impl PayloadEvent for LegendRankingsParticipationRewardEventPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::LegendRankingsParticipationReward
    }
}

// ********** BILLING ************** //

/// Payload for billing.payment.created event
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BillingPaymentCreatedPayload {
    pub payment_id: String,
    pub user_id: String,
    pub amount: i64,
    pub currency: String,
    pub status: String, // "pending" | "processing"
    pub metadata: HashMap<String, String>,
    pub occurred_at: String,
}

impl PayloadEvent for BillingPaymentCreatedPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::BillingPaymentCreated
    }
}

/// Payload for billing.payment.succeeded event
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BillingPaymentSucceededPayload {
    pub payment_id: String,
    pub user_id: String,
    pub amount: i64,
    pub currency: String,
    pub metadata: HashMap<String, String>,
    pub occurred_at: String,
}

impl PayloadEvent for BillingPaymentSucceededPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::BillingPaymentSucceeded
    }
}

/// Payload for billing.payment.failed event
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BillingPaymentFailedPayload {
    pub payment_id: String,
    pub user_id: String,
    pub amount: i64,
    pub currency: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_reason: Option<String>,
    pub metadata: HashMap<String, String>,
    pub occurred_at: String,
}

impl PayloadEvent for BillingPaymentFailedPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::BillingPaymentFailed
    }
}

/// Payload for billing.payment.refunded event
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BillingPaymentRefundedPayload {
    pub payment_id: String,
    pub user_id: String,
    pub amount: i64,
    pub refunded_amount: i64,
    pub currency: String,
    pub metadata: HashMap<String, String>,
    pub occurred_at: String,
}

impl PayloadEvent for BillingPaymentRefundedPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::BillingPaymentRefunded
    }
}

/// Payload for billing.subscription.created event
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BillingSubscriptionCreatedPayload {
    pub subscription_id: String,
    pub user_id: String,
    pub plan_id: String,
    pub plan_slug: String,
    pub status: String, // "pending" | "active" | "trialing"
    pub period_start: String,
    pub period_end: String,
    pub occurred_at: String,
}

impl PayloadEvent for BillingSubscriptionCreatedPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::BillingSubscriptionCreated
    }
}

/// Payload for billing.subscription.updated event
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BillingSubscriptionUpdatedPayload {
    pub subscription_id: String,
    pub user_id: String,
    pub plan_id: String,
    pub plan_slug: String,
    pub status: String, // "active" | "past_due" | "unpaid" | "paused" | "trialing"
    pub cancel_at_period_end: bool,
    pub period_start: String,
    pub period_end: String,
    pub occurred_at: String,
}

impl PayloadEvent for BillingSubscriptionUpdatedPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::BillingSubscriptionUpdated
    }
}

/// Payload for billing.subscription.renewed event
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BillingSubscriptionRenewedPayload {
    pub subscription_id: String,
    pub user_id: String,
    pub plan_id: String,
    pub plan_slug: String,
    pub period_start: String,
    pub period_end: String,
    pub occurred_at: String,
}

impl PayloadEvent for BillingSubscriptionRenewedPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::BillingSubscriptionRenewed
    }
}

/// Payload for billing.subscription.canceled event
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BillingSubscriptionCanceledPayload {
    pub subscription_id: String,
    pub user_id: String,
    pub plan_id: String,
    pub plan_slug: String,
    pub canceled_at: String,
    pub occurred_at: String,
}

impl PayloadEvent for BillingSubscriptionCanceledPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::BillingSubscriptionCanceled
    }
}

/// Payload for billing.subscription.expired event
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BillingSubscriptionExpiredPayload {
    pub subscription_id: String,
    pub user_id: String,
    pub plan_id: String,
    pub plan_slug: String,
    pub expired_at: String,
    pub occurred_at: String,
}

impl PayloadEvent for BillingSubscriptionExpiredPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::BillingSubscriptionExpired
    }
}

// ********** LEGEND EVENTS ************** //

/// Payload for legend_events.new_event_created event
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LegendEventsNewEventCreatedPayload {
    pub event_id: i32,
    pub title: String,
    pub description: String,
    pub author_email: String,
    pub reward_type: Option<String>,
    pub start_date: String,
    pub end_date: String,
    pub max_players: Option<i32>,
    pub ticket_price_usd: Option<f32>,
    pub is_free_tournament: bool,
    /// Optional notification config (JSON) to enrich email templates
    pub notification_config: Option<NotificationConfig>,
}

impl PayloadEvent for LegendEventsNewEventCreatedPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::LegendEventsNewEventCreated
    }
}

/// Payload for legend_events.event_started event
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LegendEventsEventStartedPayload {
    pub event_id: i32,
    pub title: String,
    pub started_at: String,
}

impl PayloadEvent for LegendEventsEventStartedPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::LegendEventsEventStarted
    }
}

/// Payload for legend_events.event_ended event
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LegendEventsEventEndedPayload {
    pub event_id: i32,
    pub title: String,
    pub ended_at: String,
    pub total_participants: i32,
}

impl PayloadEvent for LegendEventsEventEndedPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::LegendEventsEventEnded
    }
}

/// Payload for legend_events.player_registered event
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LegendEventsPlayerRegisteredPayload {
    pub event_id: i32,
    pub user_id: String,
    pub payment_id: Option<String>,
    pub amount_paid: Option<f32>,
    pub is_free: bool,
    pub registered_at: String,
}

impl PayloadEvent for LegendEventsPlayerRegisteredPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::LegendEventsPlayerRegistered
    }
}

/// Payload for legend_events.player_joined_waitlist event
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LegendEventsPlayerJoinedWaitlistPayload {
    pub event_id: i32,
    pub user_id: String,
    pub position: i32,
    pub joined_at: String,
}

impl PayloadEvent for LegendEventsPlayerJoinedWaitlistPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::LegendEventsPlayerJoinedWaitlist
    }
}

/// Payload for legend_events.score_submitted event
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LegendEventsScoreSubmittedPayload {
    pub event_id: i32,
    pub user_id: String,
    pub score: f64,
    pub total_score: f64,
    pub match_id: Option<String>,
    pub submitted_at: String,
}

impl PayloadEvent for LegendEventsScoreSubmittedPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::LegendEventsScoreSubmitted
    }
}

/// Represents a completed event with its winners
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompletedEvent {
    pub event_id: i32,
    pub title: String,
    pub description: String,
    pub author_email: String,
    pub ends_at: String,
    pub reward: Option<String>,
    pub reward_type: Option<String>,
    pub winners: Vec<EventWinner>,
    /// Optional notification config forwarded from event
    pub notification_config: Option<serde_json::Value>,
}

/// Represents a winner in an event
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EventWinner {
    pub user_id: String,
    pub position: i32,
    pub score: f64,
}

/// Payload for legend_events.events_finished event
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LegendEventsEventsFinishedPayload {
    pub completed_events: Vec<CompletedEvent>,
}

impl PayloadEvent for LegendEventsEventsFinishedPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::LegendEventsEventsFinished
    }
}

/// Payload for legend_events.intermediate_reward event
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LegendEventsIntermediateRewardPayload {
    pub user_id: String,
    pub event_id: i32,
    pub intermediate_reward_type: String,
    pub reward_config: serde_json::Value,
    pub template_name: String,
    pub template_data: serde_json::Value,
}

impl PayloadEvent for LegendEventsIntermediateRewardPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::LegendEventsIntermediateReward
    }
}

/// Payload for legend_events.participation_reward event
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LegendEventsParticipationRewardPayload {
    pub user_id: String,
    pub event_id: i32,
    pub participation_reward_type: String,
    pub reward_config: serde_json::Value,
    pub template_name: String,
    pub template_data: serde_json::Value,
}

impl PayloadEvent for LegendEventsParticipationRewardPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::LegendEventsParticipationReward
    }
}

// ********** AUDIT ************** //
/// Payload for audit.received event - tracks when event is received before processing
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuditReceivedPayload {
    /// The microservice that published the original event
    pub publisher_microservice: String,
    /// The microservice that received the event
    pub receiver_microservice: String,
    /// The event that was received
    pub received_event: String,
    /// Timestamp when the event was received (UNIX timestamp in milliseconds)
    pub received_at: u64,
    /// The queue name from which the event was consumed
    pub queue_name: String,
    /// Event identifier for cross-event correlation (UUID v7)
    pub event_id: String,
}

impl PayloadEvent for AuditReceivedPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::AuditReceived
    }
}

/// Payload for audit.processed event - tracks successful event processing
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuditProcessedPayload {
    /// The microservice that published the original event
    pub publisher_microservice: String,
    /// The microservice that processed the event
    pub processor_microservice: String,
    /// The original event that was processed
    pub processed_event: String,
    /// Timestamp when the event was processed (UNIX timestamp in milliseconds)
    pub processed_at: u64,
    /// The queue name where the event was consumed
    pub queue_name: String,
    /// Event identifier for cross-event correlation (UUID v7)
    pub event_id: String,
}

impl PayloadEvent for AuditProcessedPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::AuditProcessed
    }
}

/// Payload for audit.dead_letter event - tracks when message is rejected/nacked
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuditDeadLetterPayload {
    /// The microservice that published the original event
    pub publisher_microservice: String,
    /// The microservice that rejected the event
    pub rejector_microservice: String,
    /// The original event that was rejected
    pub rejected_event: String,
    /// Timestamp when the event was rejected (UNIX timestamp in milliseconds)
    pub rejected_at: u64,
    /// The queue name where the event was rejected from
    pub queue_name: String,
    /// Reason for rejection (delay, fibonacci_strategy, etc.)
    pub rejection_reason: String,
    /// Optional retry count
    pub retry_count: Option<u32>,
    /// Event identifier for cross-event correlation (UUID v7)
    pub event_id: String,
}

impl PayloadEvent for AuditDeadLetterPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::AuditDeadLetter
    }
}

/// Payload for audit.published event - tracks when event is published at the source microservice
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuditPublishedPayload {
    /// The microservice that published the event
    pub publisher_microservice: String,
    /// The event that was published
    pub published_event: String,
    /// Timestamp when the event was published (UNIX timestamp in milliseconds)
    pub published_at: u64,
    /// Event identifier for cross-event correlation (UUID v7)
    pub event_id: String,
}

impl PayloadEvent for AuditPublishedPayload {
    fn event_type(&self) -> MicroserviceEvent {
        MicroserviceEvent::AuditPublished
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    // Year 2020 in milliseconds (Jan 1, 2020 00:00:00 UTC)
    const YEAR_2020_MS: u64 = 1577836800000;
    // Year 2030 in milliseconds (Jan 1, 2030 00:00:00 UTC)
    const YEAR_2030_MS: u64 = 1893456000000;

    #[test]
    fn test_audit_published_payload_timestamp_precision() {
        let current_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let payload = AuditPublishedPayload {
            publisher_microservice: "test-service".to_string(),
            published_event: "test.event".to_string(),
            published_at: current_ms,
            event_id: "test-uuid".to_string(),
        };

        // Verify timestamp is in reasonable range (year 2020-2030)
        assert!(
            payload.published_at > YEAR_2020_MS,
            "Timestamp {} should be after year 2020 ({})",
            payload.published_at,
            YEAR_2020_MS
        );
        assert!(
            payload.published_at < YEAR_2030_MS,
            "Timestamp {} should be before year 2030 ({})",
            payload.published_at,
            YEAR_2030_MS
        );

        // Verify millisecond precision (should have 3+ more digits than seconds)
        let as_seconds = payload.published_at / 1000;
        assert!(
            payload.published_at > as_seconds * 1000,
            "Timestamp should have millisecond precision, not just seconds"
        );
    }

    #[test]
    fn test_audit_received_payload_timestamp_precision() {
        let current_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let payload = AuditReceivedPayload {
            publisher_microservice: "publisher-service".to_string(),
            receiver_microservice: "receiver-service".to_string(),
            received_event: "test.event".to_string(),
            received_at: current_ms,
            queue_name: "test_queue".to_string(),
            event_id: "test-uuid".to_string(),
        };

        // Verify timestamp is in reasonable range
        assert!(payload.received_at > YEAR_2020_MS);
        assert!(payload.received_at < YEAR_2030_MS);

        // Verify millisecond precision
        let as_seconds = payload.received_at / 1000;
        assert!(payload.received_at > as_seconds * 1000);
    }

    #[test]
    fn test_audit_processed_payload_timestamp_precision() {
        let current_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let payload = AuditProcessedPayload {
            publisher_microservice: "publisher-service".to_string(),
            processor_microservice: "processor-service".to_string(),
            processed_event: "test.event".to_string(),
            processed_at: current_ms,
            queue_name: "test_queue".to_string(),
            event_id: "test-uuid".to_string(),
        };

        // Verify timestamp is in reasonable range
        assert!(payload.processed_at > YEAR_2020_MS);
        assert!(payload.processed_at < YEAR_2030_MS);

        // Verify millisecond precision
        let as_seconds = payload.processed_at / 1000;
        assert!(payload.processed_at > as_seconds * 1000);
    }

    #[test]
    fn test_audit_dead_letter_payload_timestamp_precision() {
        let current_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let payload = AuditDeadLetterPayload {
            publisher_microservice: "publisher-service".to_string(),
            rejector_microservice: "rejector-service".to_string(),
            rejected_event: "test.event".to_string(),
            rejected_at: current_ms,
            queue_name: "test_queue".to_string(),
            rejection_reason: "test_reason".to_string(),
            retry_count: Some(3),
            event_id: "test-uuid".to_string(),
        };

        // Verify timestamp is in reasonable range
        assert!(payload.rejected_at > YEAR_2020_MS);
        assert!(payload.rejected_at < YEAR_2030_MS);

        // Verify millisecond precision
        let as_seconds = payload.rejected_at / 1000;
        assert!(payload.rejected_at > as_seconds * 1000);
    }

    #[test]
    fn test_millisecond_vs_second_timestamps() {
        // Generate timestamp in milliseconds
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        // Generate timestamp in seconds (old way)
        let timestamp_s = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Milliseconds should be ~1000x larger than seconds
        assert!(
            timestamp_ms > timestamp_s * 100,
            "Millisecond timestamp {} should be much larger than second timestamp {}",
            timestamp_ms,
            timestamp_s
        );

        // If treated as milliseconds, seconds would show as 1970
        // (timestamp_s in milliseconds would be < year 2000)
        assert!(
            timestamp_s < YEAR_2020_MS,
            "Second timestamp {} would be before year 2000 if treated as milliseconds",
            timestamp_s
        );
    }
}
