use bson::Document;

use crate::{BboMsg, CrawlerType, FundingRateMsg, L2EventMsg, L2SnapshotMsg, L2TopKMsg, TradeMsg};

const BIN_SIZE: f64 = 0.001;
const NEXT_BID_MULTIPLIER: f64 = 1.0 - BIN_SIZE;
const NEXT_ASK_MULTIPLIER: f64 = 1.0 + BIN_SIZE;

impl L2SnapshotMsg {
    pub fn finalize(mut self) -> Self {
        for allocation in &self.bids {
            if !self.bid_bins.is_empty() {
                let last = self.bid_bins.last().unwrap();
                if allocation[0] >= last[0] * NEXT_BID_MULTIPLIER {
                    // add to existing bin liquidity
                    self.bid_bins.last_mut().unwrap()[1] += allocation[0] * allocation[1];
                } else {
                    // create new bin
                    while allocation[0] < (self.bid_bins.last().unwrap()[0] * NEXT_BID_MULTIPLIER) {
                        let last = self.bid_bins.last().unwrap();
                        self.bid_bins.push([last[0] * NEXT_BID_MULTIPLIER, 0.0]);
                    }
                    self.bid_bins.last_mut().unwrap()[1] += allocation[0] * allocation[1];
                }
            } else {
                // this is TOB bin
                self.bid_bins
                    .push([allocation[0], allocation[0] * allocation[1]]);
            }
        }
        for allocation in &self.asks {
            if !self.ask_bins.is_empty() {
                let last = self.ask_bins.last().unwrap();
                if allocation[0] <= last[0] * NEXT_ASK_MULTIPLIER {
                    // add to existing bin size
                    self.ask_bins.last_mut().unwrap()[1] += allocation[0] * allocation[1];
                } else {
                    // create new bin
                    while allocation[0] > (self.ask_bins.last().unwrap()[0] * NEXT_ASK_MULTIPLIER) {
                        let last = self.ask_bins.last().unwrap();
                        self.ask_bins.push([last[0] * NEXT_ASK_MULTIPLIER, 0.0]);
                    }
                    self.ask_bins.last_mut().unwrap()[1] += allocation[0] * allocation[1];
                }
            } else {
                // this is TOB bin
                self.ask_bins
                    .push([allocation[0], allocation[0] * allocation[1]]);
            }
        }
        self
    }
    pub fn finalize_size_is_quote(mut self) -> Self {
        for allocation in &self.bids {
            if !self.bid_bins.is_empty() {
                let last = self.bid_bins.last().unwrap();
                if allocation[0] >= last[0] * NEXT_BID_MULTIPLIER {
                    // add to existing bin liquidity
                    self.bid_bins.last_mut().unwrap()[1] += allocation[1];
                } else {
                    // create new bin
                    while allocation[0] < (self.bid_bins.last().unwrap()[0] * NEXT_BID_MULTIPLIER) {
                        let last = self.bid_bins.last().unwrap();
                        self.bid_bins.push([last[0] * NEXT_BID_MULTIPLIER, 0.0]);
                    }
                    self.bid_bins.last_mut().unwrap()[1] += allocation[1];
                }
            } else {
                // this is TOB bin
                self.bid_bins.push([allocation[0], allocation[1]]);
            }
        }
        for allocation in &self.asks {
            if !self.ask_bins.is_empty() {
                let last = self.ask_bins.last().unwrap();
                if allocation[0] <= last[0] * NEXT_ASK_MULTIPLIER {
                    // add to existing bin size
                    self.ask_bins.last_mut().unwrap()[1] += allocation[1];
                } else {
                    // create new bin
                    while allocation[0] > (self.ask_bins.last().unwrap()[0] * NEXT_ASK_MULTIPLIER) {
                        let last = self.ask_bins.last().unwrap();
                        self.ask_bins.push([last[0] * NEXT_ASK_MULTIPLIER, 0.0]);
                    }
                    self.ask_bins.last_mut().unwrap()[1] += allocation[1];
                }
            } else {
                // this is TOB bin
                self.ask_bins.push([allocation[0], allocation[1]]);
            }
        }
        self
    }
}

pub trait HasCrawlerType {
    fn crawler_type() -> CrawlerType;
}

impl HasCrawlerType for BboMsg {
    fn crawler_type() -> CrawlerType {
        CrawlerType::Bbo
    }
}

impl HasCrawlerType for FundingRateMsg {
    fn crawler_type() -> CrawlerType {
        CrawlerType::FundingRate
    }
}

impl HasCrawlerType for L2EventMsg {
    fn crawler_type() -> CrawlerType {
        CrawlerType::L2Event
    }
}

impl HasCrawlerType for L2SnapshotMsg {
    fn crawler_type() -> CrawlerType {
        CrawlerType::L2Snapshot
    }
}

impl HasCrawlerType for L2TopKMsg {
    fn crawler_type() -> CrawlerType {
        CrawlerType::L2Topk
    }
}

impl HasCrawlerType for Document {
    fn crawler_type() -> CrawlerType {
        CrawlerType::Ticker
    }
}

impl HasCrawlerType for TradeMsg {
    fn crawler_type() -> CrawlerType {
        CrawlerType::Trade
    }
}
