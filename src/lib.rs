// why the hell is this very opinion-based lint on by default
#![allow(clippy::new_without_default)]
pub use bus::Bus;
use std::{error::Error, mem};
use std::{fmt::Debug, sync::mpsc::SendError};
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
    Self: PartialEq + Debug,
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
    fn send_event(&self, message: Self::Event) -> Result<(), SendError<Self::Event>>;
}

pub trait TerminationCondition<M: Message> {
    fn terminates(&mut self, message: &M) -> bool;
}
impl<M: Message> TerminationCondition<M> for Option<Box<dyn FnMut(&M) -> bool>> {
    fn terminates(&mut self, message: &M) -> bool {
        if let Some(c) = self {
            c(message)
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
impl<M: Message + Debug, T: FnMut(&M) -> bool> TerminationCondition<M> for T {
    fn terminates(&mut self, message: &M) -> bool {
        self(message)
    }
}
pub trait EventDistributor {
    type Event: Message;
    type Error: Error;
    type EventSender;
    type SubscriptionSender;
    type TerminationCondition: TerminationCondition<Self::Event>;
    fn serve_events(&mut self) -> Result<(), Self::Error>;
    fn get_event_sink(&self) -> Self::EventSender;
    fn get_subscrition_sink(&self) -> Self::SubscriptionSender;
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
        let tr = rx.recv();
        assert_eq!(tr, Ok(TestMessage::FooEvent));
        etx.send(TestMessage::Terminate).unwrap();
    }
}
