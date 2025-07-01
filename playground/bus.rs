use bus::Bus;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// Define our event types
#[derive(Clone, Debug)]
#[allow(dead_code)]
enum Event {
    UserJoined(String),
    MessageSent { user: String, message: String },
    UserLeft(String),
}

fn main() {
    // Create a new event bus with Mutex
    let bus: Arc<Mutex<Bus<Event>>> = Arc::new(Mutex::new(Bus::new(10)));

    // Clone the bus for our publisher
    let publisher_bus = Arc::clone(&bus);

    // Spawn a thread to publish events
    let publisher = thread::spawn(move || {
        thread::sleep(Duration::from_millis(100));
        publisher_bus
            .lock()
            .unwrap()
            .broadcast(Event::UserJoined("Alice".to_string()));
        thread::sleep(Duration::from_millis(100));
        publisher_bus.lock().unwrap().broadcast(Event::MessageSent {
            user: "Alice".to_string(),
            message: "Hello, everyone!".to_string(),
        });
        thread::sleep(Duration::from_millis(100));
        publisher_bus
            .lock()
            .unwrap()
            .broadcast(Event::UserLeft("Alice".to_string()));
    });

    // Create subscribers
    let subscriber1 = bus.lock().unwrap().add_rx();
    let subscriber2 = bus.lock().unwrap().add_rx();

    // Spawn threads for subscribers
    let handle1 = thread::spawn(move || {
        for event in subscriber1 {
            println!("Subscriber 1 received: {event:?}");
        }
    });

    let handle2 = thread::spawn(move || {
        for event in subscriber2 {
            println!("Subscriber 2 received: {event:?}");
        }
    });

    // Wait for the publisher to finish
    publisher.join().unwrap();

    // Wait a bit to ensure all events are processed
    thread::sleep(Duration::from_millis(500));

    // Drop the bus to end the subscriber loops
    drop(bus);

    // Wait for subscribers to finish
    handle1.join().unwrap();
    handle2.join().unwrap();
}
