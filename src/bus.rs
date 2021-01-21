use crate::{EventDistributor, Message, Subscription};
use std::{
    error::Error,
    sync::mpsc::{self, Receiver, Sender},
};

#[derive(Debug)]
pub enum BusError {
    Error,
}
impl std::fmt::Display for BusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as std::fmt::Debug>::fmt(self, f)
    }
}
impl Error for BusError {}

pub struct Bus<M, S>
where
    M: Message,
    S: Subscription<Event = M>,
{
    event_reciever: Receiver<M>,
    event_sender: Sender<M>,
    subscription_reciever: Receiver<S>,
    subscription_sender: Sender<S>,
    pub(crate) subscribers: Vec<S>,
}
impl<M, S> Bus<M, S>
where
    M: Message,
    S: Subscription<Event = M>,
{
    pub fn new() -> Bus<M, S> {
        let (etx, erx) = mpsc::channel();
        let (stx, srx) = mpsc::channel();

        Bus::<M, S> {
            event_reciever: erx,
            event_sender: etx,
            subscription_reciever: srx,
            subscription_sender: stx,
            subscribers: Vec::new(),
        }
    }
}
impl<M: Message, S: Subscription<Event = M>> EventDistributor for Bus<M, S> {
    type Event = M;

    type Error = BusError;

    fn serve_events(&mut self) -> Result<(), Self::Error> {
        while let Ok(subscriber) = self.subscription_reciever.try_recv() {
            self.subscribers.push(subscriber);
        }
        Ok(())
    }

    type EventSender = Sender<M>;

    type SubscriptionSender = Sender<S>;

    fn get_event_sink(&self) -> Self::EventSender {
        self.event_sender.clone()
    }

    fn get_subscrition_sink(&self) -> Self::SubscriptionSender {
        self.subscription_sender.clone()
    }
}
