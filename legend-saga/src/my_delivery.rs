use lapin::message::Delivery;
use lapin::types::{DeliveryTag, FieldTable, ShortString};

#[derive(Debug, PartialEq, Clone)]
pub struct MyDelivery {
    /// The delivery tag of the message. Use this for
    /// acknowledging the message.
    pub delivery_tag: DeliveryTag,

    /// The exchange of the message. Maybe an empty string
    /// if the default exchange is used.
    pub exchange: ShortString,

    // /// The routing key of the message. Maybe an empty string
    // /// if no routing key is specified.
    // pub routing_key: ShortString,

    // /// Whether this message was redelivered
    // pub redelivered: bool,

    // /// Contains the properties and the headers of the
    // /// message.
    // pub properties: BasicProperties,
    app_id: Option<ShortString>,
    message_id: Option<ShortString>,
    /// The payload of the message in binary format.
    pub data: Vec<u8>,
    pub headers: FieldTable,
}

impl MyDelivery {
    pub fn new(delivery: &Delivery) -> Self {
        MyDelivery {
            delivery_tag: delivery.delivery_tag,
            exchange: delivery.exchange.clone(),
            // routing_key: delivery.routing_key.clone(),
            // redelivered: delivery.redelivered,
            headers: delivery.properties.headers().clone().unwrap_or_default(),
            // properties: delivery.properties.clone(),
            data: delivery.data.clone(),
            app_id: delivery.properties.app_id().to_owned(),
            message_id: delivery.properties.message_id().to_owned()
        }
    }
    pub fn app_id(&self) -> &Option<ShortString> {
        &self.app_id
    }

    pub fn message_id(&self) -> &Option<ShortString> {
        &self.message_id
    }
    
    pub fn with_app_id(mut self, value: ShortString) -> Self {
        self.app_id = Some(value);
        self
    }

    pub fn with_message_id(mut self, value: ShortString) -> Self {
        self.message_id = Some(value);
        self
    }
}
