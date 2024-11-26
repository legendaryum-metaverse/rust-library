use std::collections::HashMap;
use std::future::Future;
use std::hash::Hash;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

pub struct Emitter<T, U>
where
    T: Clone + Send + 'static,
    U: Eq + Hash + Clone + Send + 'static,
{
    events: Arc<Mutex<HashMap<U, mpsc::Sender<T>>>>,
}

impl<T, U> Emitter<T, U>
where
    T: Clone + Send + 'static,
    U: Eq + Hash + Clone + Send + 'static,
{
    pub(crate) fn clone(&self) -> Self {
        Emitter {
            events: self.events.clone(),
        }
    }
}

impl<T, U> Default for Emitter<T, U>
where
    T: Clone + Send + 'static,
    U: Eq + Hash + Clone + Send + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, U> Emitter<T, U>
where
    T: Clone + Send + 'static,
    U: Eq + Hash + Clone + Send + 'static,
{
    pub(crate) fn new() -> Self {
        Emitter {
            events: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn on(&self, event: U) -> mpsc::Receiver<T> {
        let mut events = self.events.lock().await;
        let (tx, rx) = mpsc::channel(100); // Buffer size of 100, adjust as needed
        events.entry(event).or_insert(tx);
        rx
    }

    pub async fn on_with_async_handler<F, Fut>(&self, event: U, mut handler: F)
    where
        F: FnMut(T) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let mut rx = self.on(event).await;
        tokio::spawn(async move {
            while let Some(data) = rx.recv().await {
                handler(data).await;
            }
        });
    }

    pub(crate) async fn emit(&self, event: U, data: T) {
        let events = self.events.lock().await;
        if let Some(sender) = events.get(&event) {
            let _ = sender.send(data).await;
        }
    }
}

#[cfg(test)]
mod test_emitter {
    use crate::emitter::Emitter;
    use crate::events::MicroserviceEvent;
    use std::fmt::Debug;
    use std::sync::Arc;

    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::sync::Notify;
    use tokio::time::{timeout, Duration};

    #[derive(Clone, Debug)]
    struct EventHandler {
        payload: String,
    }

    #[tokio::test]
    async fn simple_emitter() {
        let emitter = Emitter::<EventHandler, MicroserviceEvent>::new();

        let notify_shutdown = Arc::new(Notify::new());

        // Subscribe to an event with an async handler
        let notify_shutdown_clone = notify_shutdown.clone();
        emitter
            .on_with_async_handler(MicroserviceEvent::AuthDeletedUser, move |handler| {
                let n = notify_shutdown_clone.clone();
                async move {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    assert_eq!(handler.payload, "my_payload");
                    n.notify_one();
                }
            })
            .await;

        // Emit an event
        let event_data = EventHandler {
            payload: "my_payload".to_string(),
        };
        emitter
            .emit(MicroserviceEvent::AuthDeletedUser, event_data)
            .await;

        notify_shutdown.notified().await;
    }

    #[derive(Clone, Debug, PartialEq)]
    struct EventPayload {
        id: usize,
        data: String,
    }

    #[derive(Clone, Debug, Eq, Hash, PartialEq)]
    enum TestEvent {
        Event1,
        Event2,
        Event3,
    }

    #[tokio::test]
    async fn test_basic_emit_and_receive() {
        let emitter = Emitter::<EventPayload, TestEvent>::new();
        let notify = Arc::new(Notify::new());
        let notify_clone = notify.clone();

        emitter
            .on_with_async_handler(TestEvent::Event1, move |payload| {
                let n = notify_clone.clone();
                async move {
                    assert_eq!(payload.id, 1);
                    assert_eq!(payload.data, "test data");
                    n.notify_one();
                }
            })
            .await;

        emitter
            .emit(
                TestEvent::Event1,
                EventPayload {
                    id: 1,
                    data: "test data".to_string(),
                },
            )
            .await;

        timeout(Duration::from_secs(1), notify.notified())
            .await
            .expect("Timed out waiting for event");
    }

    /// Only the first handler declared is stored in the emitter with a link "TestEvent::Event2"
    /// consequent calls to on_with_async_handler don't store/update the handler
    #[tokio::test]
    async fn test_multiple_handlers() {
        let emitter = Emitter::<EventPayload, TestEvent>::new();
        let counter = Arc::new(AtomicUsize::new(0));

        for i in 0..3 {
            let counter_clone = counter.clone();
            emitter
                .on_with_async_handler(TestEvent::Event2, move |_| {
                    let c = counter_clone.clone();
                    async move {
                        c.store(i + 1, Ordering::SeqCst);
                    }
                })
                .await;
        }

        emitter
            .emit(
                TestEvent::Event2,
                EventPayload {
                    id: 2,
                    data: "multi handler".to_string(),
                },
            )
            .await;

        tokio::time::sleep(Duration::from_millis(100)).await;
        assert_eq!(counter.load(Ordering::SeqCst), 1); // only i==0 + 1  -> 1 -> first iteration
    }

    #[tokio::test]
    async fn test_different_event_types() {
        let emitter = Emitter::<EventPayload, TestEvent>::new();
        let notify1 = Arc::new(Notify::new());
        let notify2 = Arc::new(Notify::new());

        let n1 = notify1.clone();
        emitter
            .on_with_async_handler(TestEvent::Event1, move |payload| {
                let n = n1.clone();
                async move {
                    assert_eq!(payload.id, 1);
                    n.notify_one();
                }
            })
            .await;

        let n2 = notify2.clone();
        emitter
            .on_with_async_handler(TestEvent::Event2, move |payload| {
                let n = n2.clone();
                async move {
                    assert_eq!(payload.id, 2);
                    n.notify_one();
                }
            })
            .await;

        emitter
            .emit(
                TestEvent::Event1,
                EventPayload {
                    id: 1,
                    data: "event 1".to_string(),
                },
            )
            .await;
        emitter
            .emit(
                TestEvent::Event2,
                EventPayload {
                    id: 2,
                    data: "event 2".to_string(),
                },
            )
            .await;

        timeout(Duration::from_secs(1), notify1.notified())
            .await
            .expect("Timed out waiting for event 1");
        timeout(Duration::from_secs(1), notify2.notified())
            .await
            .expect("Timed out waiting for event 2");
    }

    #[tokio::test]
    async fn test_emitter_clone() {
        let emitter1 = Emitter::<EventPayload, TestEvent>::new();
        let emitter2 = emitter1.clone();

        let notify = Arc::new(Notify::new());
        let notify_clone = notify.clone();

        emitter1
            .on_with_async_handler(TestEvent::Event3, move |payload| {
                let n = notify_clone.clone();
                async move {
                    assert_eq!(payload.id, 3);
                    assert_eq!(payload.data, "cloned emitter");
                    n.notify_one();
                }
            })
            .await;

        emitter2
            .emit(
                TestEvent::Event3,
                EventPayload {
                    id: 3,
                    data: "cloned emitter".to_string(),
                },
            )
            .await;

        timeout(Duration::from_secs(1), notify.notified())
            .await
            .expect("Timed out waiting for event from cloned emitter");
    }

    #[tokio::test]
    async fn test_unhandled_event() {
        let emitter = Emitter::<EventPayload, TestEvent>::new();

        // Emit an event with no handlers
        emitter
            .emit(
                TestEvent::Event1,
                EventPayload {
                    id: 1,
                    data: "unhandled".to_string(),
                },
            )
            .await;

        // If we reached this point without panicking, the test passes
    }
}
