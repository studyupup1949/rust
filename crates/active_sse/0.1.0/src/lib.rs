/*
 * MIT License (MIT)
 * Copyright (c) 2019 Activeledger
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

//! # Activeledger SSE Helper
//!
//! This crate is a helper crate that handles the setup and configuration of Server Sent Events
//! (Eventsource) calls to Activeledger.
//!
//! ## Usage
//!
//! To create a new ActiveSSE instance you need to create a config first. This defines various
//! data needed ActiveSSE to connect to the correct Activeledger API endpoint.
//!
//! The config types are Activity and Event. These are created simply by calling
//!
//! `let mut config = Config::activity("http://base-url");`
//!
//! `let mut config = Config::event("http://base-url");`
//!
//! Once you have created the initial config instance you can pass in the optional data via
//! function calls.
//!
//! Once configured, use it to create a new instance of ActiveSSE:
//! `let sse = ActiveSSE::new(config);`
//!
//! When you are ready, call the subscribe function:
//! `let receiver = sse.subscribe();`
//!
//! Then you can listen for activity or events
//! ```
//! # use active_sse::{Config, ActiveSSE};
//! # let config = Config::activity("http://localhost:5260");
//! # let listener = ActiveSSE::new(config);
//! # let receiver = listener.subscribe().unwrap();
//! // Note that this will only return once
//! println!("{:?}", receiver.recv().unwrap());
//! ```
//!
//! This returns a [`receiever`](https://doc.rust-lang.org/nightly/std/sync/mpsc/struct.Receiver.html)
//! which makes up part of Rust's channel type. You can retrieve messages to it by calling
//! [`recv`](https://doc.rust-lang.org/nightly/std/sync/mpsc/struct.Receiver.html#method.recv)
//!
//! This will only be triggered once, but can be put into a loop to listen for multiple events.
//!
//! ### A complete setup might look like this
//! ```
//! use active_sse::{Config, ActiveSSE};
//!
//! let mut config = Config::activity("http://localhost:5260");
//!
//! config.set_stream_id("Stream id").unwrap();
//!
//! let listener = ActiveSSE::new(config);
//! let receiver = listener.subscribe().unwrap();
//!
//! // Note that this will only return once
//! println!("{:?}", receiver.recv().unwrap());
//! ```
//!
//! ## Additional Activeledger crates
//! Adhearing to the Rust mentality of keeping things small we have created other crates that can be used in conjunction
//! with this one to add additional functionality.
//!
//! These crates are:
//! * [activeledger](https://github.com/activeledger/SDK-Rust) - For handling connections and keys ([Crate](https://crates.io/crates/activeledger))
//! * [active_txbuilder](https://github.com/activeledger/SDK-Rust-TxBuilder) - To build transactions without worrying about the JSON. ([Crate](https://crates.io/crates/active_tx))
//!
//! ## Links
//!
//! [Activeledger](https://activeledger.io)
//!
//! [Activeledger Developers portal](https://developers.activeledger.io)
//!
//! [Activeledger on GitHub](https://github.com/activeledger/activeledger)
//!
//! [Activeledger on NPM](https://www.npmjs.com/package/@activeledger/activeledger)
//!
//! [This SDK on GitHub](https://github.com/activeledger/SDK-Rust)
//!
//! [Report Issues](https://github.com/activeledger/SDK-Rust/issues)

pub mod error;

extern crate sse_client;
use sse_client::{Event, EventSource};

use std::sync::mpsc::Receiver;

use error::{SSEError, SSEResult};

pub struct ActiveSSE {
    config: Config,
}

#[derive(PartialEq)]
enum ConfigType {
    Activity,
    Event,
}

pub struct Config {
    url: String,
    config_type: ConfigType,
    stream_id: Option<String>,
    contract: Option<String>,
    event: Option<String>,
}

impl Config {
    /// # Create an activity config
    ///
    /// This function creates a new activity configuration that points to a specified URL.
    /// Subscribing to an activity will listen for stream creation and update activity.
    /// It can be used as is to listen globally or it can be given a specific stream to listen
    /// for updates.
    ///
    /// ## Listen globally
    /// ```
    /// # use active_sse::Config;
    /// let mut config = Config::activity("http://localhost:5260");
    /// ```
    ///
    /// ## Listen to a specific stream
    /// ```
    /// # use active_sse::Config;
    /// let mut config = Config::activity("http://localhost:5260");
    ///
    /// config.set_stream_id("stream id");
    /// ```
    pub fn activity(url: &str) -> Config {
        // Append the path to the url
        let url = format!("{}/api/activity/subscribe/", url);

        Config {
            url,
            config_type: ConfigType::Activity,
            stream_id: None,
            contract: None,
            event: None,
        }
    }

    /// # Create an event config
    ///
    /// This function creates a new event configuration that points to a specified URL.
    /// This configuration can be used as is to listen for all events on a network, it can be
    /// given a contract to listen to all events triggered by that contract, or it can be given
    /// both a contract and an event to listen to.
    ///
    /// ## Listen globally
    /// ```
    /// # use active_sse::Config;
    /// let mut config = Config::activity("http://localhost:5260");
    /// ```
    ///
    /// /// ## Listen to all events from a contract
    /// ```
    /// # use active_sse::Config;
    /// let mut config = Config::event("http://localhost:5260");
    ///
    /// config.set_contract("contract id").unwrap();
    /// ```
    ///
    /// ## Listen for a specific event on a contract
    /// ```
    /// # use active_sse::Config;
    /// let mut config = Config::event("http://localhost:5260");
    ///
    /// config.set_contract("contract_id").unwrap();
    /// config.set_event("event").unwrap();
    /// ```
    pub fn event(url: &str) -> Config {
        // Append the path to the url
        let url = format!("{}/api/events/", url);

        Config {
            url,
            config_type: ConfigType::Event,
            stream_id: None,
            contract: None,
            event: None,
        }
    }

    /// # Listen for activity on a specific stream
    ///
    /// This function sets the stream id stored in a confuiguration object. It can be used to
    /// listen for any activity on a specific stream.
    ///
    /// ```
    /// # use active_sse::Config;
    /// let mut config = Config::activity("http://localhost:5260");
    ///
    /// config.set_stream_id("stream id").unwrap();
    /// ```
    pub fn set_stream_id(&mut self, id: &str) -> SSEResult<&mut Self> {
        // Check that this is an Activity config
        if ConfigType::Activity != self.config_type {
            return Err(SSEError::IncompatibleConfig);
        }

        self.stream_id = Some(id.to_owned());
        self.url = format!("{}{}", self.url, id);

        Ok(self)
    }

    /// # Listen for events on a specific contract
    ///
    /// This function sets the contract id stored in a confuiguration object. It can be used to
    /// listen for any events on a specific contract.
    ///
    /// ```
    /// # use active_sse::Config;
    /// let mut config = Config::event("http://localhost:5260");
    ///
    /// config.set_contract("contract id").unwrap();
    /// ```
    pub fn set_contract(&mut self, contract: &str) -> SSEResult<&mut Self> {
        // Check that this is an Event config
        if ConfigType::Event != self.config_type {
            return Err(SSEError::IncompatibleConfig);
        }

        self.contract = Some(contract.to_owned());
        self.url = format!("{}{}", self.url, contract);

        Ok(self)
    }

    /// # Listen for a specific event on a specific contract
    ///
    /// This function sets the event name stored in a confuiguration object. It can be used to
    /// listen for a specific events on a specific contract.
    ///
    /// A contract must be set before an event name can be provided.
    ///
    /// ```
    /// # use active_sse::Config;
    /// let mut config = Config::event("http://localhost:5260");
    ///
    /// config.set_contract("contract id").unwrap();
    /// config.set_event("event").unwrap();
    /// ```
    pub fn set_event(&mut self, event: &str) -> SSEResult<&mut Self> {
        // Check that this is an Event config
        if ConfigType::Event != self.config_type {
            return Err(SSEError::IncompatibleConfig);
        }

        // Check that a contract has been set
        if self.contract.is_none() {
            return Err(SSEError::ContractNotSet);
        }

        self.event = Some(event.to_owned());
        self.url = format!("{}{}", self.url, event);

        Ok(self)
    }

    /// Used to get the URL from the config
    fn get_url(&self) -> &str {
        &self.url
    }
}

impl ActiveSSE {
    /// # Create a new ActiveSSE instance using a given config
    ///
    /// ## Listen for activities
    /// ```
    /// # use active_sse::{Config, ActiveSSE};
    /// let config = Config::activity("http://localhost:5260");
    ///
    /// let listener = ActiveSSE::new(config);
    ///
    /// let receiver = listener.subscribe().unwrap();
    ///
    /// // Note that this will only be called once
    /// println!("{:?}", receiver.recv().unwrap());
    /// ```
    ///
    /// ## Listen for events
    /// ```
    /// # use active_sse::{Config, ActiveSSE};
    /// let config = Config::event("http://localhost:5260");
    ///
    /// let listener = ActiveSSE::new(config);
    ///
    /// let receiver = listener.subscribe().unwrap();
    ///
    /// // Note that this will only return once
    /// println!("{:?}", receiver.recv().unwrap());
    /// ```
    pub fn new(config: Config) -> ActiveSSE {
        ActiveSSE { config }
    }

    /// # Subscribe to a listener
    ///
    /// ## Listen for activities
    /// ```
    /// # use active_sse::{Config, ActiveSSE};
    /// let config = Config::activity("http://localhost:5260");
    ///
    /// let listener = ActiveSSE::new(config);
    ///
    /// let receiver = listener.subscribe().unwrap();
    ///
    /// // Note that this will only return once
    /// println!("{:?}", receiver.recv().unwrap());
    /// ```
    ///
    /// ## Listen for events
    /// ```
    /// # use active_sse::{Config, ActiveSSE};
    /// let config = Config::event("http://localhost:5260");
    ///
    /// let listener = ActiveSSE::new(config);
    ///
    /// let receiver = listener.subscribe().unwrap();
    ///
    /// // Note that this will only return once
    /// println!("{:?}", receiver.recv().unwrap());
    /// ```
    pub fn subscribe(&self) -> SSEResult<Receiver<Event>> {
        let client = match EventSource::new(self.config.get_url()) {
            Ok(client) => client,
            Err(_) => return Err(SSEError::EventSource),
        };

        Ok(client.receiver())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    #[ignore]
    fn it_works() {
        let config = Config::activity("http://testnet-uk.activeledger.io:5261");

        let sse = ActiveSSE::new(config);

        let rec = sse.subscribe().unwrap();

        loop {
            println!("{:?}", rec.recv().unwrap());
        }
    }
}
