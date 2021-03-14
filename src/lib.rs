pub use bus::Bus;
use std::{error::Error, mem};
use std::{fmt::Debug, sync::mpsc::SendError};
pub use subscriber::Subscriber;
mod bus;
mod subscriber;

/// A trait that describes a message that can be seent through a bus
pub trait Message
where
    Self: Clone,
{
    type Discriminant: Discriminant<Self>;
    fn discriminant(&self) -> Self::Discriminant;
}

/**
A trait that describes a value that message variants can be compared by
# Examples

```
use evelyn::Message
use evelyn::Discriminant
// For enums Discriminant trait is implemented for `std::mem::Discriminant`
enum TestMessage {
   A,
   B,
}
impl Message for TestMessage {
    type Discriminant = std::mem::Discriminant<Self>;
    fn discriminant(&self) -> Self::Discriminant {
        std::mem::discriminant(self)
    }
}
// Example implementation for strings using first 5 characters as discriminant
impl Message for String {
    type Discriminant = [char; 5];
    fn discriminant(&self) -> Self::Discriminant {
        self.chars.take(5).collect()
    }
}
```
*/
pub trait Discriminant<M>
where
    Self: PartialEq + Debug,
{
}
impl<T> Discriminant<T> for mem::Discriminant<T> {}

/// A trait describin a bus subscription
pub trait Subscription
where
    Self: PartialEq,
{
    type Event: Message;
    /// Test whether this subscriiption is subscribed to a message
    fn subscribed_to(&self, message: &Self::Event) -> bool;
    /// Returns a slice of discriminants corresponding to messages this subsccription is subscribed to
    fn discriminant_set(&self) -> &[<Self::Event as Message>::Discriminant];
    /// Delivers a message to this subscription's reciever
    fn send_event(&self, message: Self::Event) -> Result<(), SendError<Self::Event>>;
}

/// Trait describing a condition in which message should terminate the bus' event loop
pub trait TerminationCondition<M: Message> {
    fn terminates(&mut self, message: &M) -> bool;
}
impl<M: Message, T: TerminationCondition<M>> TerminationCondition<M> for Option<T> {
    //Option<Box<dyn FnMut(&M) -> bool>> {
    fn terminates(&mut self, message: &M) -> bool {
        if let Some(c) = self {
            c.terminates(message)
        } else {
            false
        }
    }
}
impl<M: Message> TerminationCondition<M> for () {
    fn terminates(&mut self, _message: &M) -> bool {
        false
    }
}
impl<M: Message, F: FnMut(&M) -> bool> TerminationCondition<M> for F {
    fn terminates(&mut self, message: &M) -> bool {
        self(message)
    }
}

/// Trait describing an event bus
pub trait EventDistributor {
    type Event: Message;
    type Error: Error;
    type EventSender;
    type SubscriptionSender;
    type TerminationCondition: TerminationCondition<Self::Event>;
    /// Entry point to the event loop
    fn serve_events(&mut self) -> Result<(), Self::Error>;
    fn get_event_sink(&self) -> Self::EventSender;
    fn get_subscrition_sink(&self) -> Self::SubscriptionSender;
}

mod foo {
    use crate::{Discriminant, Message};
    // For enum based messages, Discriminant trait is implemented for `std::mem::Discriminant`
    #[derive(Clone)]
    enum TestMessage {
        A,
        B,
    }
    impl Message for TestMessage {
        type Discriminant = std::mem::Discriminant<Self>;
        fn discriminant(&self) -> Self::Discriminant {
            std::mem::discriminant(self)
        }
    }
    impl Discriminant<String> for [char; 5] {}
    // Example implementation for strings using first 5 characters as discriminant
    impl Message for String {
        type Discriminant = [char; 5];
        fn discriminant(&self) -> Self::Discriminant {
            use std::convert::TryInto;
            self.chars().take(5).collect::<Vec<_>>().try_into().unwrap()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{mem::discriminant, sync::mpsc, thread};
    type TestBus = Bus<TestMessage, Subscriber<TestMessage>, fn(&TestMessage) -> bool>;
    fn terminator(msg: &TestMessage) -> bool {
        matches!(msg, TestMessage::Terminate)
    }
    #[derive(Clone, PartialEq, Debug)]
    enum TestMessage {
        FooEvent,
        Terminate,
    }
    impl Message for TestMessage {
        type Discriminant = mem::Discriminant<Self>;
        fn discriminant(&self) -> Self::Discriminant {
            discriminant(self)
        }
    }

    #[test]
    fn can_accept_subscribers() {
        let mut bus: TestBus = Bus::new(terminator);
        assert!(bus.subscribers.is_empty());
        let subscribe_sink = bus.get_subscrition_sink();
        let (tx, _) = mpsc::channel();
        let subscriber = Subscriber::<TestMessage> {
            sender: tx,
            discriminant_set: vec![],
        };
        let etx = bus.get_event_sink();
        etx.send(TestMessage::Terminate).unwrap();
        subscribe_sink.send(subscriber).unwrap();
        bus.serve_events().unwrap();
        assert_eq!(bus.subscribers.len(), 1);
    }
    #[test]
    fn subscriber_recieves_events() {
        let mut bus: TestBus = Bus::new(terminator);
        let subscribe_sink = bus.get_subscrition_sink();
        let (tx, rx) = mpsc::channel();
        let subscriber = Subscriber::<TestMessage> {
            sender: tx,
            discriminant_set: vec![TestMessage::FooEvent.discriminant()],
        };
        let etx = bus.get_event_sink();

        etx.send(TestMessage::FooEvent).unwrap();
        subscribe_sink.send(subscriber).unwrap();

        thread::spawn(move || bus.serve_events().unwrap());
        assert_eq!(rx.recv(), Ok(TestMessage::FooEvent));
        etx.send(TestMessage::Terminate).unwrap();
    }

    // This could be blanket impl'd for Message, but alas that would conflict with blanket impl for FnMut(M) -> bool
    impl TerminationCondition<TestMessage> for TestMessage {
        fn terminates(&mut self, message: &TestMessage) -> bool {
            self == message
        }
    }
    #[test]
    fn bus_terminated_by_message() {
        let mut bus: Bus<TestMessage, Subscriber<TestMessage>, TestMessage> =
            Bus::new(TestMessage::Terminate);
        let subscribe_sink = bus.get_subscrition_sink();
        let (tx, rx) = mpsc::channel();
        let subscriber = Subscriber::<TestMessage> {
            sender: tx,
            discriminant_set: vec![TestMessage::FooEvent.discriminant()],
        };
        let etx = bus.get_event_sink();

        etx.send(TestMessage::FooEvent).unwrap();
        subscribe_sink.send(subscriber).unwrap();

        thread::spawn(move || bus.serve_events().unwrap());
        assert_eq!(rx.recv(), Ok(TestMessage::FooEvent));
        etx.send(TestMessage::Terminate).unwrap();
    }
}
