use futures::Sink;
use futures::unsync::mpsc::UnboundedSender;
use shell_words::ParseError;
use std::any::{Any, TypeId};
use std::cell::{RefCell, RefMut, Ref};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::hash;
use std::io::Error as IoError;
use std::rc::Rc;
use std::string::FromUtf8Error;
use tokio_xmpp::Packet;
use uuid::Uuid;
use xmpp_parsers::{Element, FullJid, BareJid, Jid};
use xmpp_parsers;
use chrono::{Utc, DateTime};

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub from: BareJid,
    pub from_full: Jid,
    pub to: BareJid,
    pub to_full: Jid,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct GroupchatMessage {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub from: BareJid,
    pub from_full: Jid,
    pub to: BareJid,
    pub to_full: Jid,
    pub body: String,
}

#[derive(Debug, Clone)]
pub enum XmppMessage {
    Chat(ChatMessage),
    Groupchat(GroupchatMessage),
}

#[derive(Debug, Clone)]
pub struct LogMessage {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub body: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    Incoming(XmppMessage),
    Outgoing(XmppMessage),
    Log(LogMessage),
}

impl Message {
    pub fn incoming_chat<I: Into<String>>(id: I, timestamp: DateTime<Utc>, from_full: &Jid, to_full: &Jid, body: &str) -> Self {
        let from = match from_full {
            Jid::Bare(from_full) => from_full.clone(),
            Jid::Full(from_full) => from_full.clone().into(),
        };

        let to = match to_full {
            Jid::Bare(to_full) => to_full.clone(),
            Jid::Full(to_full) => to_full.clone().into(),
        };

        Message::Incoming(XmppMessage::Chat(ChatMessage {
            id: id.into(),
            timestamp: timestamp,
            from: from,
            from_full: from_full.clone(),
            to: to.clone(),
            to_full: to_full.clone(),
            body: body.to_string(),
        }))
    }

    pub fn outgoing_chat<I: Into<String>>(id: I, timestamp: DateTime<Utc>, from_full: &Jid, to_full: &Jid, body: &str) -> Self {
        let from = match from_full {
            Jid::Bare(from_full) => from_full.clone(),
            Jid::Full(from_full) => from_full.clone().into(),
        };

        let to = match to_full {
            Jid::Bare(to_full) => to_full.clone(),
            Jid::Full(to_full) => to_full.clone().into(),
        };

        Message::Outgoing(XmppMessage::Chat(ChatMessage {
            id: id.into(),
            timestamp: timestamp,
            from: from,
            from_full: from_full.clone(),
            to: to.clone(),
            to_full: to_full.clone(),
            body: body.to_string(),
        }))
    }

    pub fn incoming_groupchat<I: Into<String>>(id: I, timestamp: DateTime<Utc>, from_full: &Jid, to_full: &Jid, body: &str) -> Self {
        let from = match from_full {
            Jid::Bare(from_full) => from_full.clone(),
            Jid::Full(from_full) => from_full.clone().into(),
        };

        let to = match to_full {
            Jid::Bare(to_full) => to_full.clone(),
            Jid::Full(to_full) => to_full.clone().into(),
        };

        Message::Incoming(XmppMessage::Groupchat(GroupchatMessage {
            id: id.into(),
            timestamp: timestamp,
            from: from,
            from_full: from_full.clone(),
            to: to.clone(),
            to_full: to_full.clone(),
            body: body.to_string(),
        }))
    }

    pub fn outgoing_groupchat<I: Into<String>>(id: I, timestamp: DateTime<Utc>, from_full: &Jid, to_full: &Jid, body: &str) -> Self {
        let from = match from_full {
            Jid::Bare(from_full) => from_full.clone(),
            Jid::Full(from_full) => from_full.clone().into(),
        };

        let to = match to_full {
            Jid::Bare(to_full) => to_full.clone(),
            Jid::Full(to_full) => to_full.clone().into(),
        };

        Message::Outgoing(XmppMessage::Groupchat(GroupchatMessage {
            id: id.into(),
            timestamp: timestamp,
            from: from,
            from_full: from_full.clone(),
            to: to.clone(),
            to_full: to_full.clone(),
            body: body.to_string(),
        }))
    }

    pub fn log(msg: String) -> Self {
        Message::Log(LogMessage {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            body: msg
        })
    }

    #[allow(dead_code)]
    pub fn body(&self) -> &str {
        match self {
            Message::Outgoing(XmppMessage::Chat(ChatMessage { body, .. }))
                | Message::Incoming(XmppMessage::Chat(ChatMessage { body, .. }))
                | Message::Outgoing(XmppMessage::Groupchat(GroupchatMessage { body, .. }))
                | Message::Incoming(XmppMessage::Groupchat(GroupchatMessage { body, .. }))
                | Message::Log(LogMessage { body, .. }) => &body,
        }
    }
}

impl hash::Hash for Message {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        match self {
            Message::Log(message) => message.id.hash(state),
            Message::Incoming(XmppMessage::Chat(message))
                | Message::Outgoing(XmppMessage::Chat(message)) => message.id.hash(state),
            Message::Incoming(XmppMessage::Groupchat(message))
                | Message::Outgoing(XmppMessage::Groupchat(message)) => message.id.hash(state),
        }
    }
}

impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        let my_id = match self {
            Message::Log(message) => &message.id,
            Message::Incoming(XmppMessage::Chat(message))
                | Message::Outgoing(XmppMessage::Chat(message)) => &message.id,
            Message::Incoming(XmppMessage::Groupchat(message))
                | Message::Outgoing(XmppMessage::Groupchat(message)) => &message.id,
        };

        let other_id = match other {
            Message::Log(message) => &message.id,
            Message::Incoming(XmppMessage::Chat(message))
                | Message::Outgoing(XmppMessage::Chat(message)) => &message.id,
            Message::Incoming(XmppMessage::Groupchat(message))
                | Message::Outgoing(XmppMessage::Groupchat(message)) => &message.id,
        };

        my_id == other_id
    }
}

impl std::cmp::Eq for Message {
}

//impl TryFrom<xmpp_parsers::Message> for Message {
//}

impl TryFrom<Message> for xmpp_parsers::Element {
    type Error = ();

    fn try_from(message: Message) -> Result<Self, Self::Error> {
        match message {
            Message::Log(_) => {
                Err(())
            },
            Message::Incoming(_) => {
                Err(())
            },
            Message::Outgoing(XmppMessage::Chat(message)) => {
                let mut xmpp_message = xmpp_parsers::message::Message::new(Some(Jid::Bare(message.to)));
                xmpp_message.id = Some(message.id);
                xmpp_message.type_ = xmpp_parsers::message::MessageType::Chat;
                xmpp_message.bodies.insert(String::new(), xmpp_parsers::message::Body(message.body));
                Ok(xmpp_message.into())
            },
            Message::Outgoing(XmppMessage::Groupchat(message)) => {
                let mut xmpp_message = xmpp_parsers::message::Message::new(Some(Jid::Bare(message.to)));
                xmpp_message.id = Some(message.id);
                xmpp_message.type_ = xmpp_parsers::message::MessageType::Groupchat;
                xmpp_message.bodies.insert(String::new(), xmpp_parsers::message::Body(message.body));
                Ok(xmpp_message.into())
            }
        }
    }
}

pub enum CommandOrMessage {
    Command(Command),
    Message(Message),
}

#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub args: Vec<String>,
}

impl Command {
    pub fn new(command: String, args: Vec<String>) -> Self {
        Self {
            name: command,
            args: args,
        }
    }
}

#[derive(Debug, Error)]
pub enum CommandError {
    Io(IoError),
    Utf8(FromUtf8Error),
    Parse(ParseError),
}

pub enum Event {
    Connected(FullJid),
    #[allow(dead_code)]
    Disconnected(FullJid),
    Message(Message),
    Join(FullJid),
}

pub trait Plugin: fmt::Display {
    fn new() -> Self where Self: Sized;
    fn init(&mut self, mgr: &Aparte) -> Result<(), ()>;
    fn on_event(&mut self, aparte: Rc<Aparte>, event: &Event);
}

pub trait AnyPlugin: Any + Plugin {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn as_plugin(&mut self) -> &mut dyn Plugin;
}

impl<T> AnyPlugin for T where T: Any + Plugin {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_plugin(&mut self) -> &mut dyn Plugin {
        self
    }
}

pub struct Connection {
    pub sink: UnboundedSender<Packet>,
    pub account: FullJid,
}

pub struct Aparte {
    commands: HashMap<String, fn(Rc<Aparte>, &Command) -> Result<(), ()>>,
    plugins: HashMap<TypeId, RefCell<Box<dyn AnyPlugin>>>,
    connections: RefCell<HashMap<String, Connection>>,
    current_connection: RefCell<Option<String>>,
}

impl Aparte {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
            plugins: HashMap::new(),
            connections: RefCell::new(HashMap::new()),
            current_connection: RefCell::new(None),
        }
    }

    pub fn add_command(&mut self, name: &str, command: fn(Rc<Aparte>, &Command) -> Result<(), ()>) {
        self.commands.insert(name.to_string(), command);
    }

    pub fn parse_command(self: Rc<Self>, command: &Command) -> Result<(), ()> {
        match self.commands.get(&command.name) {
            Some(parser) => parser(self, command),
            None => Err(()),
        }
    }

    pub fn add_plugin<T: 'static>(&mut self, plugin: Box<dyn AnyPlugin>) -> Result<(), ()> {
        info!("Add plugin `{}`", plugin);
        self.plugins.insert(TypeId::of::<T>(), RefCell::new(plugin));
        Ok(())
    }

    pub fn get_plugin<T: 'static>(&self) -> Option<Ref<T>> {
        let rc = match self.plugins.get(&TypeId::of::<T>()) {
            Some(rc) => rc,
            None => return None,
        };

        let any_plugin = rc.borrow();
        /* Calling unwrap here on purpose as we expect panic if plugin is not of the right type */
        Some(Ref::map(any_plugin, |p| p.as_any().downcast_ref::<T>().unwrap()))
    }

    pub fn get_plugin_mut<T: 'static>(&self) -> Option<RefMut<T>> {
        let rc = match self.plugins.get(&TypeId::of::<T>()) {
            Some(rc) => rc,
            None => return None,
        };

        let any_plugin = rc.borrow_mut();
        /* Calling unwrap here on purpose as we expect panic if plugin is not of the right type */
        Some(RefMut::map(any_plugin, |p| p.as_any_mut().downcast_mut::<T>().unwrap()))
    }

    pub fn add_connection(&self, account: FullJid, sink: UnboundedSender<Packet>) {
        let connection = Connection {
            account: account,
            sink: sink,
        };

        let account = connection.account.to_string();

        self.connections.borrow_mut().insert(account.clone(), connection);
        self.current_connection.replace(Some(account.clone()));
    }

    pub fn current_connection(&self) -> Option<FullJid> {
        let current_connection = self.current_connection.borrow();
        match &*current_connection {
            Some(current_connection) => {
                let connections = self.connections.borrow_mut();
                let connection = connections.get(&current_connection.clone()).unwrap();
                Some(connection.account.clone())
            },
            None => None,
        }
    }

    pub fn init(&mut self) -> Result<(), ()> {
        for (_, plugin) in self.plugins.iter() {
            if let Err(err) = plugin.borrow_mut().as_plugin().init(&self) {
                return Err(err);
            }
        }

        Ok(())
    }

    pub fn send(&self, element: Element) {
        debug!("SEND: {:?}", element);
        let packet = Packet::Stanza(element);
        // TODO use correct connection
        let mut connections = self.connections.borrow_mut();
        let current_connection = connections.iter_mut().next().unwrap().1;
        let mut sink = &current_connection.sink;
        if let Err(e) = sink.start_send(packet) {
            warn!("Cannot send packet: {}", e);
        }
    }

    pub fn event(self: Rc<Self>, event: Event) {
        for (_, plugin) in self.plugins.iter() {
            plugin.borrow_mut().as_plugin().on_event(Rc::clone(&self), &event);
        }
    }

    pub fn log(self: Rc<Self>, message: String) {
        let message = Message::log(message);
        self.event(Event::Message(message));
    }
}
