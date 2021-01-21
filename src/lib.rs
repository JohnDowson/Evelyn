// why the hell is this very opinion-based lint on by default
#![allow(clippy::new_without_default)]
pub use bus::Bus;
use std::sync::mpsc::SendError;
use std::{error::Error, mem};
pub use subscriber::Subscriber;
mod bus;
mod subscriber;

pub trait Message
where
    Self: Clone,
{
    type Discriminant: Discriminant<Self>;
    fn discriminant(&self) -> Self::Discriminant;
}
pub trait Discriminant<M>
where
    Self: PartialEq,
{
}
impl<T> Discriminant<T> for mem::Discriminant<T> {}
pub trait Subscription
where
    Self: PartialEq,
{
    type Event: Message;
    fn subscribed_to(&self, message: &Self::Event) -> bool;
    fn discriminant_set(&self) -> &[<Self::Event as Message>::Discriminant];
    fn send_event(&mut self, message: Self::Event) -> Result<(), SendError<Self::Event>>;
}
pub trait EventDistributor {
    type Event: Message + Clone;
    type Error: Error;
    type EventSender;
    type SubscriptionSender;
    fn serve_events(&mut self) -> Result<(), Self::Error>;
    fn get_event_sink(&self) -> Self::EventSender;
    fn get_subscrition_sink(&self) -> Self::SubscriptionSender;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{mem::discriminant, sync::mpsc};

    #[derive(Clone, PartialEq)]
    enum TestMessage {
        FooEvent,
        BarEvent,
    }
    impl Message for TestMessage {
        type Discriminant = mem::Discriminant<Self>;
        fn discriminant(&self) -> Self::Discriminant {
            discriminant(self)
        }
    }

    #[test]
    fn can_accept_subscribers() {
        let mut bus: Bus<TestMessage, Subscriber<TestMessage>> = Bus::new();
        assert!(bus.subscribers.is_empty());
        let subscribe_sink = bus.get_subscrition_sink();
        let (tx, _) = mpsc::channel();
        let subscriber = Subscriber::<TestMessage> {
            sender: tx,
            discriminant_set: vec![],
        };
        subscribe_sink.send(subscriber).unwrap();
        bus.serve_events().unwrap();
        assert_eq!(bus.subscribers.len(), 1);
    }
}
