use xmpp_parsers::roster::Subscription;
use std::hash::{Hash, Hasher};
use xmpp_parsers::BareJid;

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum Presence {
    Unavailable,
    Available,
    Away,
    Chat,
    Dnd,
    Xa,
}

#[derive(Clone, Debug)]
pub struct Group(pub String);

impl Hash for Group {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl PartialEq for Group {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Group {}

#[derive(Clone, Debug)]
pub struct Contact {
    pub jid: BareJid,
    pub name: Option<String>,
    pub subscription: Subscription,
    pub presence: Presence,
    pub groups: Vec<Group>,
}

impl Hash for Contact {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.jid.hash(state);
    }
}

impl PartialEq for Contact {
    fn eq(&self, other: &Self) -> bool {
        self.jid == other.jid
    }
}

impl Eq for Contact {}
