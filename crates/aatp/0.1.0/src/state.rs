//! Protocol state machine — the layer that validates a *sequence* of frames, not
//! just each frame. A session walks `Handshake → Active → Closed`; anything out of
//! order (a data frame before the handshake, traffic after close) is rejected, so
//! two agents can never drift out of sync.
//!
//! The transitions are written with explicit `==`/`&&` rather than a `match` so the
//! prober has real logic to attack: every rule below is pinned by an exhaustive
//! test over all `State × Event` pairs, so *no* mutation of a transition can slip a
//! frame into an illegal state without a test going red. That exhaustiveness is the
//! "formal verification" here — mutation-completeness of the transition relation.

/// Where a session is in its lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// Initial: awaiting the peer's handshake before any data may flow.
    Handshake,
    /// Handshake complete: data frames are allowed.
    Active,
    /// Session ended: nothing further is accepted.
    Closed,
}

/// The kind of frame driving a transition (the application maps a frame's type byte
/// onto one of these).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// The opening handshake.
    Hello,
    /// A data / result frame.
    Data,
    /// An orderly close.
    Bye,
}

/// A rejected transition — the session stays in its current state, unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IllegalTransition {
    /// The state the session was in.
    pub state: State,
    /// The event that was not legal from that state.
    pub event: Event,
}

impl State {
    /// A fresh session begins awaiting the handshake.
    pub fn start() -> State {
        State::Handshake
    }

    /// Apply `event`, returning the next state — or [`IllegalTransition`] (leaving the
    /// caller's state untouched) if the sequence would be out of order. The three legal
    /// edges are: `Handshake --Hello--> Active`, `Active --Data--> Active`, and
    /// `Active --Bye--> Closed`. Everything else — a `Data` before the handshake, a
    /// second `Hello`, any frame after `Closed` — is refused.
    pub fn on(self, event: Event) -> Result<State, IllegalTransition> {
        // Two edges lead to `Active` — the opening handshake, and a data frame while
        // already active — so they share one guard (both blocks were identical). The
        // explicit `==`/`&&`/`||` keep the transition relation a rich target for the
        // prober; the exhaustive test below pins every cell regardless.
        let to_active = (self == State::Handshake && event == Event::Hello)
            || (self == State::Active && event == Event::Data);
        if to_active {
            Ok(State::Active)
        } else if self == State::Active && event == Event::Bye {
            Ok(State::Closed)
        } else {
            Err(IllegalTransition { state: self, event })
        }
    }

    /// Whether the session has ended.
    pub fn is_closed(self) -> bool {
        self == State::Closed
    }

    /// Whether data frames are legal right now (only in `Active`).
    pub fn accepts_data(self) -> bool {
        self == State::Active
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const STATES: [State; 3] = [State::Handshake, State::Active, State::Closed];
    const EVENTS: [Event; 3] = [Event::Hello, Event::Data, Event::Bye];

    /// The expected next state for every `(state, event)` pair; `None` = illegal. This
    /// table IS the specification — the exhaustive check below pins all nine cells, so a
    /// mutation of any `==`/`&&` in `on` flips at least one cell and is caught.
    fn expected(state: State, event: Event) -> Option<State> {
        match (state, event) {
            (State::Handshake, Event::Hello) => Some(State::Active),
            (State::Active, Event::Data) => Some(State::Active),
            (State::Active, Event::Bye) => Some(State::Closed),
            _ => None,
        }
    }

    #[test]
    fn every_transition_matches_the_specification_table() {
        for &s in &STATES {
            for &e in &EVENTS {
                match expected(s, e) {
                    Some(next) => assert_eq!(
                        s.on(e),
                        Ok(next),
                        "legal {s:?} --{e:?}--> {next:?} must be allowed"
                    ),
                    None => assert_eq!(
                        s.on(e),
                        Err(IllegalTransition { state: s, event: e }),
                        "illegal {s:?} --{e:?}--> must be refused"
                    ),
                }
            }
        }
    }

    #[test]
    fn start_is_handshake_and_a_normal_session_reaches_closed() {
        let s = State::start();
        assert_eq!(s, State::Handshake);
        let s = s.on(Event::Hello).unwrap();
        assert_eq!(s, State::Active);
        let s = s.on(Event::Data).unwrap();
        assert_eq!(s, State::Active); // data keeps it Active
        let s = s.on(Event::Bye).unwrap();
        assert_eq!(s, State::Closed);
    }

    #[test]
    fn data_before_handshake_is_the_classic_out_of_sync_bug_and_is_refused() {
        assert_eq!(
            State::start().on(Event::Data),
            Err(IllegalTransition {
                state: State::Handshake,
                event: Event::Data
            })
        );
    }

    #[test]
    fn nothing_is_accepted_after_close_no_deadlock_no_reopen() {
        let closed = State::Closed;
        for &e in &EVENTS {
            assert!(closed.on(e).is_err(), "{e:?} after close must be refused");
        }
    }

    #[test]
    fn is_closed_and_accepts_data_track_the_state_exactly() {
        assert!(!State::Handshake.is_closed());
        assert!(!State::Active.is_closed());
        assert!(State::Closed.is_closed());

        assert!(!State::Handshake.accepts_data());
        assert!(State::Active.accepts_data());
        assert!(!State::Closed.accepts_data());
    }
}
