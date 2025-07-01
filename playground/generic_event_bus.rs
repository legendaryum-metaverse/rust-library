use std::collections::HashMap;
use std::future::Future;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, Notify};

pub struct Emitter<T, U>
where
    T: Clone + Send + 'static,
    U: Eq + Hash + Clone + Send + 'static,
{
    events: Arc<Mutex<HashMap<U, mpsc::Sender<T>>>>,
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
    pub fn new() -> Self {
        Emitter {
            events: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn on(&self, event: U) -> mpsc::Receiver<T> {
        let mut events = self.events.lock().unwrap();
        let (tx, rx) = mpsc::channel(100); // Buffer size of 100, adjust as needed
        events.entry(event).or_insert(tx);
        rx
    }

    pub fn on_with_async_handler<F, Fut>(&self, event: U, mut handler: F)
    where
        F: FnMut(T) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let mut rx = self.on(event);
        tokio::spawn(async move {
            while let Some(data) = rx.recv().await {
                handler(data).await;
            }
        });
    }

    pub async fn emit(&self, event: U, data: T) {
        let events = self.events.lock().unwrap().clone();
        if let Some(sender) = events.get(&event) {
            let _ = sender.send(data).await;
        }
    }
}

// Example usage
use serde_json::Value;
use std::fmt::Debug;

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct EventHandler {
    channel: String,
    payload: HashMap<String, Value>,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
enum MicroserviceEvent {
    AuthDeletedUserEvent,
    // Add other events as needed
}

#[tokio::main]
async fn main() {
    let emitter = Emitter::<EventHandler, MicroserviceEvent>::new();

    let notify_shutdown = Arc::new(Notify::new());

    // emitter.on_with_async_handler(MicroserviceEvent::AuthDeletedUserEvent, |handler| async move {
    //
    //     println!("Received AuthDeletedUserEvent: {:?}", handler);
    //     // Simulate some async operation
    //     tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    //     println!("Finished processing AuthDeletedUserEvent");
    // });

    // Subscribe to an event with an async handler
    let notify_shutdown_2 = notify_shutdown.clone();
    emitter.on_with_async_handler(MicroserviceEvent::AuthDeletedUserEvent, move |handler| {
        let notify_shutdown_clone = notify_shutdown_2.clone();
        async move {
            println!("Received AuthDeletedUserEvent: {handler:?}");
            // Simulate some async operation
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            println!("Finished processing AuthDeletedUserEvent");
            notify_shutdown_clone.notify_one();
        }
    });

    // Emit an event
    let event_data = EventHandler {
        channel: "auth_channel".to_string(),
        payload: HashMap::new(),
    };
    emitter
        .emit(MicroserviceEvent::AuthDeletedUserEvent, event_data)
        .await;

    //block
    notify_shutdown.notified().await;
}
