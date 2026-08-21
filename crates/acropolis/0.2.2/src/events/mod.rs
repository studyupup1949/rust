use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

pub const LARGE_EVENT_BUFFER_SIZE: usize = 100_000;
pub const SMALL_EVENT_BUFFER_SIZE: usize = 1_024;
pub const LOCAL_EVENT_BUFFER_SIZE: usize = 1_000;
pub const EVENT_WORKERS: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EventName(String);

impl EventName {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for EventName {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for EventName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum EventPayload {
    #[default]
    None,
    Text(String),
    Number(u64),
    Bytes(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub name: EventName,
    pub when: SystemTime,
    pub payload: EventPayload,
}

impl Event {
    pub fn new(name: impl Into<EventName>, payload: EventPayload) -> Self {
        Self {
            name: name.into(),
            when: SystemTime::now(),
            payload,
        }
    }
}

type EventSubscriber = Arc<dyn Fn(&Event) + Send + Sync + 'static>;
pub type SubscriberId = u64;

#[derive(Clone)]
struct SubscriberEntry {
    id: SubscriberId,
    subscriber: EventSubscriber,
}

#[derive(Default)]
struct EventBusInner {
    next_id: SubscriberId,
    subscribers: HashMap<EventName, Vec<SubscriberEntry>>,
    stopped: bool,
}

#[derive(Clone, Default)]
pub struct EventBus {
    inner: Arc<Mutex<EventBusInner>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn listen(
        &self,
        name: impl Into<EventName>,
        subscriber: impl Fn(&Event) + Send + Sync + 'static,
    ) -> Result<SubscriberId, EventError> {
        let mut inner = self.inner.lock().map_err(|_| EventError::Poisoned)?;
        if inner.stopped {
            return Err(EventError::Stopped);
        }
        inner.next_id = inner.next_id.saturating_add(1);
        let id = inner.next_id;
        inner
            .subscribers
            .entry(name.into())
            .or_default()
            .push(SubscriberEntry {
                id,
                subscriber: Arc::new(subscriber),
            });
        Ok(id)
    }

    pub fn unsubscribe(&self, id: SubscriberId) -> Result<bool, EventError> {
        let mut inner = self.inner.lock().map_err(|_| EventError::Poisoned)?;
        let mut removed = false;
        for subscribers in inner.subscribers.values_mut() {
            let old_len = subscribers.len();
            subscribers.retain(|entry| entry.id != id);
            removed |= subscribers.len() != old_len;
        }
        Ok(removed)
    }

    pub fn publish(&self, event: Event) -> Result<usize, EventError> {
        let subscribers = {
            let inner = self.inner.lock().map_err(|_| EventError::Poisoned)?;
            if inner.stopped {
                return Err(EventError::Stopped);
            }
            inner
                .subscribers
                .get(&event.name)
                .cloned()
                .unwrap_or_default()
        };
        let delivered = subscribers.len();
        for entry in subscribers {
            (entry.subscriber)(&event);
        }
        Ok(delivered)
    }

    pub fn has_subscribers(&self, name: impl Into<EventName>) -> bool {
        let name = name.into();
        self.inner
            .lock()
            .map(|inner| {
                inner
                    .subscribers
                    .get(&name)
                    .is_some_and(|subscribers| !subscribers.is_empty())
            })
            .unwrap_or(false)
    }

    pub fn stop(&self) -> Result<(), EventError> {
        let mut inner = self.inner.lock().map_err(|_| EventError::Poisoned)?;
        inner.stopped = true;
        inner.subscribers.clear();
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventError {
    Stopped,
    Poisoned,
}

impl fmt::Display for EventError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stopped => f.write_str("event bus stopped"),
            Self::Poisoned => f.write_str("event bus mutex poisoned"),
        }
    }
}

impl std::error::Error for EventError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn publish_delivers_to_local_subscribers_only() {
        let bus = EventBus::new();
        let count = Arc::new(AtomicUsize::new(0));
        let seen = Arc::clone(&count);
        bus.listen("chain.new", move |_| {
            seen.fetch_add(1, Ordering::SeqCst);
        })
        .unwrap();

        let delivered = bus
            .publish(Event::new("chain.new", EventPayload::Number(42)))
            .unwrap();

        assert_eq!(delivered, 1);
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn stop_prevents_late_ringing() {
        let bus = EventBus::new();
        bus.listen("mempool.add", |_| {}).unwrap();
        bus.stop().unwrap();
        assert_eq!(
            bus.publish(Event::new("mempool.add", EventPayload::None)),
            Err(EventError::Stopped)
        );
    }
}
