macro_rules! cfg_std {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "std")]
            $item
        )*
    };
}

cfg_std! {
    pub mod commence_saga;
    mod consumers;
    mod emitter;
    mod fibo;
    mod my_delivery;
    mod nack;
    pub mod operation;
    mod publish_event;
    mod queue_consumer_props;
    pub mod saga;
    mod start;
    pub mod events_consume;
    pub mod connection;
}

#[cfg(feature = "events")]
pub mod events;

#[cfg(feature = "grpc")]
pub mod grpc;

#[cfg(test)]
mod test;
