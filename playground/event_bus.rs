use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Event trait
trait Event: Any + Send + Sync {}

// Implement Event for common types
impl Event for String {}
impl Event for i32 {}

// Custom events
#[derive(Clone, Debug)]
struct MintImageEvent {
    image_id: String,
    user: String,
}
impl Event for MintImageEvent {}

#[derive(Clone, Debug)]
struct BuyImageEvent {
    image_id: String,
    buyer: String,
    price: f64,
}
impl Event for BuyImageEvent {}

// EventBus struct
type Listeners = Vec<Arc<Mutex<dyn Fn(&dyn Any) + Send + Sync>>>; // List of listeners for an event
struct EventBus {
    listeners: HashMap<String, Listeners>,
}

impl EventBus {
    fn new() -> Self {
        EventBus {
            listeners: HashMap::new(),
        }
    }

    fn on<E: Event + 'static>(
        &mut self,
        event_name: &str,
        callback: impl Fn(&E) + Send + Sync + 'static,
    ) {
        let listeners = self.listeners.entry(event_name.to_string()).or_default();
        listeners.push(Arc::new(Mutex::new(move |event: &dyn Any| {
            if let Some(event) = event.downcast_ref::<E>() {
                callback(event);
            }
        })));
    }

    fn emit<E: Event>(&self, event_name: &str, event: E) {
        if let Some(listeners) = self.listeners.get(event_name) {
            for listener in listeners {
                listener.lock().unwrap()(&event);
            }
        }
    }
}

fn main() {
    let mut event_bus = EventBus::new();

    // Subscribe to MintImageEvent
    event_bus.on("mint_image", |event: &MintImageEvent| {
        println!("Image minted: {} by user {}", event.image_id, event.user);
    });

    // Subscribe to BuyImageEvent
    event_bus.on("buy_image", |event: &BuyImageEvent| {
        println!(
            "Image {} bought by {} for ${}",
            event.image_id, event.buyer, event.price
        );
    });

    // Emit events
    event_bus.emit(
        "mint_image",
        MintImageEvent {
            image_id: "IMG001".to_string(),
            user: "Alice".to_string(),
        },
    );

    event_bus.emit(
        "buy_image",
        BuyImageEvent {
            image_id: "IMG001".to_string(),
            buyer: "Bob".to_string(),
            price: 1.5,
        },
    );

    // You can also emit events with primitive types
    event_bus.on("simple_event", |message: &String| {
        println!("Received simple event: {}", message);
    });

    event_bus.emit("simple_event", "Hello, World!".to_string());
}
