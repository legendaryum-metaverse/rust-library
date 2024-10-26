use crate::events::MicroserviceEvent;
use crate::queue_consumer_props::{Exchange, QueueConsumerProps};
use lapin::options::ExchangeBindOptions;
use lapin::types::AMQPValue;
use lapin::{
    options::{ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions},
    types::FieldTable,
    ExchangeKind,
};
use strum::IntoEnumIterator;
use crate::connection::RabbitMQClient;

impl RabbitMQClient {
    pub(crate) async fn create_header_consumers(
        &self,
        queue_name: &str,
        events: &[MicroserviceEvent],
    ) -> Result<(), lapin::Error> {
        let channel = self.events_channel.lock().await;
        let requeue_queue = format!("{}_matching_requeue", queue_name);

        // Assert exchanges
        channel
            .exchange_declare(
                Exchange::MATCHING,
                ExchangeKind::Headers,
                ExchangeDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await?;

        channel
            .exchange_declare(
                Exchange::MATCHING_REQUEUE,
                ExchangeKind::Headers,
                ExchangeDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await?;

        // Assert queues
        channel
            .queue_declare(
                queue_name,
                QueueDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await?;

        let mut requeue_args = FieldTable::default();
        requeue_args.insert(
            "x-dead-letter-exchange".into(),
            AMQPValue::LongString(Exchange::MATCHING.into()),
        );

        channel
            .queue_declare(
                &requeue_queue,
                QueueDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                requeue_args,
            )
            .await?;

        for event in MicroserviceEvent::iter() {
            let event_str = event.as_ref();

            let mut header_event = FieldTable::default();
            header_event.insert(
                event_str.to_uppercase().into(),
                AMQPValue::LongString(event_str.into()),
            );

            // Assert event exchange and bind to Matching exchange
            channel
                .exchange_declare(
                    event_str,
                    ExchangeKind::Headers,
                    ExchangeDeclareOptions {
                        durable: true,
                        ..Default::default()
                    },
                    FieldTable::default(),
                )
                .await?;

            let mut bind_args = header_event.clone();
            bind_args.insert("all-micro".into(), AMQPValue::LongString("yes".into()));
            bind_args.insert("x-match".into(), AMQPValue::LongString("all".into()));

            channel
                .exchange_bind(
                    event_str,
                    Exchange::MATCHING,
                    "",
                    ExchangeBindOptions::default(),
                    bind_args,
                )
                .await?;

            // Assert requeue exchange and bind to MatchingRequeue exchange
            let requeue_exchange = format!("{}_requeue", event_str);
            channel
                .exchange_declare(
                    &requeue_exchange,
                    ExchangeKind::Headers,
                    ExchangeDeclareOptions {
                        durable: true,
                        ..Default::default()
                    },
                    FieldTable::default(),
                )
                .await?;

            channel
                .exchange_bind(
                    &requeue_exchange,
                    Exchange::MATCHING_REQUEUE,
                    "",
                    ExchangeBindOptions::default(),
                    header_event.clone(),
                )
                .await?;

            let mut headers_args = header_event.clone();
            headers_args.insert("micro".into(), AMQPValue::LongString(queue_name.into()));
            headers_args.insert("x-match".into(), AMQPValue::LongString("all".into()));

            if events.contains(&event) {
                // Bind queue to event exchange
                channel
                    .queue_bind(
                        queue_name,
                        event_str,
                        "",
                        QueueBindOptions::default(),
                        header_event.clone(),
                    )
                    .await?;

                // Bind requeue queue to event requeue exchange
                channel
                    .queue_bind(
                        &requeue_queue,
                        &requeue_exchange,
                        "",
                        QueueBindOptions::default(),
                        headers_args.clone(),
                    )
                    .await?;

                // Assert and bind microservice-specific exchange
                let micro_event_exchange = format!("{}_{}", event_str, queue_name);
                channel
                    .exchange_declare(
                        &micro_event_exchange,
                        ExchangeKind::Headers,
                        ExchangeDeclareOptions {
                            durable: true,
                            ..Default::default()
                        },
                        FieldTable::default(),
                    )
                    .await?;

                channel
                    .exchange_bind(
                        &micro_event_exchange,
                        Exchange::MATCHING,
                        "",
                        ExchangeBindOptions::default(),
                        headers_args.clone(),
                    )
                    .await?;

                channel
                    .queue_bind(
                        queue_name,
                        &micro_event_exchange,
                        "",
                        QueueBindOptions::default(),
                        headers_args,
                    )
                    .await?;
            } else {
                // Unbind queue from event exchange if not in events list
                channel
                    .queue_unbind(queue_name, event_str, "", header_event)
                    .await?;

                channel
                    .queue_unbind(&requeue_queue, &requeue_exchange, "", headers_args.clone())
                    .await?;

                let micro_event_exchange = format!("{}_{}", event_str, queue_name);
                channel
                    .exchange_delete(
                        &micro_event_exchange,
                        lapin::options::ExchangeDeleteOptions {
                            if_unused: false,
                            ..Default::default()
                        },
                    )
                    .await?;
            }
        }

        Ok(())
    }
    pub(crate) async fn create_consumers(
        &self,
        consumers: Vec<QueueConsumerProps>,
    ) -> Result<(), lapin::Error> {
        let channel = self.saga_channel.lock().await;

        for consumer in consumers {
            let queue_name = &consumer.queue_name;
            let exchange = &consumer.exchange;
            let requeue_queue = format!("{}_requeue", queue_name);
            let routing_key = format!("{}_routing_key", queue_name);

            // Assert exchange and queue for the consumer
            channel
                .exchange_declare(
                    exchange,
                    ExchangeKind::Direct,
                    ExchangeDeclareOptions {
                        durable: true,
                        ..ExchangeDeclareOptions::default()
                    },
                    FieldTable::default(),
                )
                .await?;

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

            channel
                .queue_bind(
                    queue_name,
                    exchange,
                    &routing_key,
                    QueueBindOptions::default(),
                    FieldTable::default(),
                )
                .await?;

            // Set up requeue mechanism
            channel
                .exchange_declare(
                    Exchange::REQUEUE,
                    ExchangeKind::Direct,
                    ExchangeDeclareOptions {
                        durable: true,
                        ..ExchangeDeclareOptions::default()
                    },
                    FieldTable::default(),
                )
                .await?;

            let mut requeue_args = FieldTable::default();
            requeue_args.insert(
                "x-dead-letter-exchange".into(),
                AMQPValue::LongString(exchange.to_string().into()),
            );

            channel
                .queue_declare(
                    &requeue_queue,
                    QueueDeclareOptions {
                        durable: true,
                        ..QueueDeclareOptions::default()
                    },
                    requeue_args,
                )
                .await?;

            channel
                .queue_bind(
                    &requeue_queue,
                    Exchange::REQUEUE,
                    &routing_key,
                    QueueBindOptions::default(),
                    FieldTable::default(),
                )
                .await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test_consumers {
    use super::*;
    use crate::test::setup::TestSetup;

    #[test]
    fn create_consumers() {
        let setup = TestSetup::new(None);

        let consumers = vec![QueueConsumerProps {
            queue_name: "my_cool_microservice".to_string(), // related to the name of the micro
            exchange: Exchange::COMMANDS,
        }];

        setup.rt.block_on(async {
            let result = setup.client.create_consumers(consumers).await;
            assert!(
                result.is_ok(),
                "Failed to create consumers: {:?}",
                result.err()
            );
            let conn = setup.client.current_connection().await.expect("Cannot get the connection").read().await;
            let t = conn.topology();

            // verifying exchanges
            let known_exchanges = vec![Exchange::COMMANDS, Exchange::REQUEUE];
            let exchanges: Vec<String> = t.exchanges.iter().map(|e| e.name.to_string()).collect();
            for exchange in known_exchanges {
                assert!(
                    exchanges.contains(&exchange.to_string()),
                    "Exchange {} not found",
                    exchange
                );
            }

            // verifying queues
            let know_queues = vec!["my_cool_microservice", "my_cool_microservice_requeue"];
            let queues: Vec<String> = t.queues.iter().map(|q| q.name.to_string()).collect();
            for queue in know_queues {
                assert!(
                    queues.contains(&queue.to_string()),
                    "Queue {} not found",
                    queue
                );
            }
        });
    }
    #[test]
    fn create_header_consumers() {
        let setup = TestSetup::new(None);

        setup.rt.block_on(async {
            let events = vec![
                MicroserviceEvent::TestImage,
                MicroserviceEvent::AuthDeletedUser,
            ];

            {
                let result = setup
                    .client
                    .create_header_consumers("my_cool_micro", &events)
                    .await;

                assert!(
                    result.is_ok(),
                    "Failed to create header consumers: {:?}",
                    result.err()
                );

                let known_queues = vec!["my_cool_micro", "my_cool_micro_matching_requeue"];
                // there are more, but those are related to my micro
                let known_exchanges = vec![
                    "auth.deleted_user_my_cool_micro",
                    "test.image_my_cool_micro",
                ];

                let conn = setup.client.current_connection().await.expect("Cannot get the connection").read().await;
                let t = conn.topology();

                // verifying exchanges
                let exchanges: Vec<String> =
                    t.exchanges.iter().map(|e| e.name.to_string()).collect();
                for exchange in known_exchanges {
                    assert!(
                        exchanges.contains(&exchange.to_string()),
                        "Exchange {} not found",
                        exchange
                    );
                }

                // verifying queues
                let queues: Vec<String> = t.queues.iter().map(|q| q.name.to_string()).collect();
                for queue in known_queues {
                    assert!(
                        queues.contains(&queue.to_string()),
                        "Queue {} not found",
                        queue
                    );
                }
            }

            // Start again only wih the event  MicroserviceEvent::TestImage, MicroserviceEvent::AuthDeletedUser is deleted from the exchanges
            let events = vec![MicroserviceEvent::TestImage];

            {
                let result = setup
                    .client
                    .create_header_consumers("my_cool_micro", &events)
                    .await;

                assert!(
                    result.is_ok(),
                    "Failed to create header consumers: {:?}",
                    result.err()
                );

                let conn = setup.client.current_connection().await.expect("Cannot get the connection").read().await;
                let t = conn.topology();

                let known_queues = vec!["my_cool_micro", "my_cool_micro_matching_requeue"];
                // verifying queues
                let queues: Vec<String> = t.queues.iter().map(|q| q.name.to_string()).collect();
                for queue in known_queues {
                    assert!(
                        queues.contains(&queue.to_string()),
                        "Queue {} not found",
                        queue
                    );
                }
                // verifying exchanges
                let exchanges: Vec<String> =
                    t.exchanges.iter().map(|e| e.name.to_string()).collect();
                // verify that the exchange related to the event AuthDeletedUser is deleted
                assert!(
                    !exchanges.contains(&"auth.deleted_user_my_cool_micro".to_string()),
                    "Exchange auth.deleted_user_my_cool_micro found"
                );
                // verify that the exchange related to the event TestImage is still there
                assert!(
                    exchanges.contains(&"test.image_my_cool_micro".to_string()),
                    "Exchange test.image_my_cool_micro not found"
                );
            }
        });
    }
}
