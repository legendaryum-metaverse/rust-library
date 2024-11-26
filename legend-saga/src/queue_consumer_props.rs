/// Represents the names of specific message queues in the RabbitMQ context.
pub struct Queue;

impl Queue {
    /// Queue used for sending replies in response to saga events.
    pub const REPLY_TO_SAGA: &'static str = "reply_to_saga";
    /// Queue used for commencing a saga.
    pub const COMMENCE_SAGA: &'static str = "commence_saga";
}

/// Represents the names of exchanges, which act as message routing hubs in the RabbitMQ context.
pub struct Exchange;

impl Exchange {
    /// Exchange dedicated to requeueing messages that require further processing in a saga process
    pub const REQUEUE: &'static str = "requeue_exchange";
    /// Exchange for sending command messages to various consumers in a saga process
    pub const COMMANDS: &'static str = "commands_exchange";
    /// Exchange used for starting a saga.
    pub const MATCHING: &'static str = "matching_exchange";
    /// Exchange dedicated to requeueing messages that require further processing.
    pub const MATCHING_REQUEUE: &'static str = "matching_requeue_exchange";
}

/// Represents the names of specific message queues in the RabbitMQ context.
pub type ExchangeType = &'static str;

/// Properties defining a queue consumer within the RabbitMQ context.
pub struct QueueConsumerProps {
    /// The name of the queue that messages will be consumed from.
    pub queue_name: String,
    /// The associated exchange for the queue, used for routing messages.
    pub exchange: ExchangeType,
}
