use crate::{EventDistributor, Message, Subscription, TerminationCondition};
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

/// A generic implementation of an event bus using `std::sync::mpsc`
pub struct Bus<M, S, C>
where
    M: Message,
    S: Subscription<Event = M>,
    C: TerminationCondition<M>,
{
    pub(crate) event_reciever: Receiver<M>,
    event_sender: Sender<M>,
    subscription_reciever: Receiver<S>,
    subscription_sender: Sender<S>,
    termination_condition: C,
    pub(crate) subscribers: Vec<S>,
}
impl<M, S, C> Bus<M, S, C>
where
    M: Message,
    S: Subscription<Event = M>,
    C: TerminationCondition<M>,
{
    pub fn new(cond: C) -> Self {
        let (etx, erx) = mpsc::channel();
        let (stx, srx) = mpsc::channel();

        Self {
            event_reciever: erx,
            event_sender: etx,
            subscription_reciever: srx,
            subscription_sender: stx,
            termination_condition: cond,
            subscribers: Vec::new(),
        }
    }
}
impl<M, S, C> EventDistributor for Bus<M, S, C>
where
    M: Message + std::fmt::Debug,
    S: Subscription<Event = M> + std::fmt::Debug,
    C: TerminationCondition<M>,
{
    type Event = M;
    type Error = BusError;
    type EventSender = Sender<M>;
    type SubscriptionSender = Sender<S>;
    type TerminationCondition = C;
    fn serve_events(&mut self) -> Result<(), Self::Error> {
        'outer: loop {
            while let Ok(subscriber) = self.subscription_reciever.try_recv() {
                self.subscribers.push(subscriber);
            }
            while let Ok(message) = self.event_reciever.try_recv() {
                if self.termination_condition.terminates(&message) {
                    break 'outer;
                };

                let mut should_remove = vec![];
                for (idx, subscriber) in self
                    .subscribers
                    .iter()
                    .filter(|s| s.subscribed_to(&message))
                    .enumerate()
                {
                    if subscriber.send_event(message.clone()).is_err() {
                        should_remove.push(idx)
                    }
                }
                for idx in should_remove {
                    self.subscribers.remove(idx);
                }
            }
        }
        Ok(())
    }

    fn get_event_sink(&self) -> Self::EventSender {
        self.event_sender.clone()
    }

    fn get_subscrition_sink(&self) -> Self::SubscriptionSender {
        self.subscription_sender.clone()
    }
}
