use crate::L2SnapshotMsg;

const BIN_SIZE: f64 = 0.001;
const NEXT_BID_MULTIPLIER: f64 = 1.0 - BIN_SIZE;
const NEXT_ASK_MULTIPLIER: f64 = 1.0 + BIN_SIZE;

impl L2SnapshotMsg {
    pub fn finalize(mut self) -> Self {
        for allocation in &self.bids {
            if self.bid_bins.len() > 0 {
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
            if self.ask_bins.len() > 0 {
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
            if self.bid_bins.len() > 0 {
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
            if self.ask_bins.len() > 0 {
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
