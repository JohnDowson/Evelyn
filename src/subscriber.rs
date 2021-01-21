use crate::{Message, Subscription};
use std::{
    fmt::Debug,
    sync::mpsc::{SendError, Sender},
};

#[derive(Debug)]
pub struct Subscriber<M>
where
    M: Message + Debug,
{
    pub(crate) sender: Sender<<Self as Subscription>::Event>,
    pub(crate) discriminant_set: Vec<<M as Message>::Discriminant>,
}
impl<M> PartialEq for Subscriber<M>
where
    M: Message + Debug,
{
    fn eq(&self, other: &Self) -> bool {
        self.discriminant_set.eq(&other.discriminant_set)
    }
}
impl<M> Subscription for Subscriber<M>
where
    M: Message + Debug,
{
    type Event = M;
    fn subscribed_to(&self, message: &Self::Event) -> bool {
        self.discriminant_set().contains(&message.discriminant())
    }

    fn discriminant_set(&self) -> &[<Self::Event as Message>::Discriminant] {
        &self.discriminant_set
    }

    fn send_event(&self, message: Self::Event) -> Result<(), SendError<Self::Event>> {
        self.sender.send(message)
    }
}
