#![feature(external_doc)]

use ta_common::traits::Indicator;
use ta_common::fixed_queue::FixedQueue;
use ema_rs::EMA;

#[doc(include = "../docs/adline.md")]
pub struct ADLine {
    accumulator: f64
}

impl ADLine {
    pub fn new() -> ADLine {
        Self {
            accumulator: 0_f64,
        }
    }
    fn calc(&mut self, high: f64, low: f64, close: f64, volume: f64) -> f64 {
        let current = (((close - low) - (high - close)) / (high - low)) * volume;
        self.accumulator = self.accumulator + current;
        return self.accumulator;
    }
}

impl Indicator<[f64; 4], f64> for ADLine {
    fn next(&mut self, input: [f64; 4]) -> f64 {
        return self.calc(input[0], input[1], input[2], input[3]);
    }

    fn reset(&mut self) {
        self.accumulator = 0_f64;
    }
}
#[doc(include = "../docs/adosc.md")]
pub struct ADOscillator {
    short_history: FixedQueue<f64>,
    long_history: FixedQueue<f64>,
    short_ad_line: ADLine,
    long_ad_line: ADLine,
    short_ema: EMA,
    long_ema: EMA,
}

impl ADOscillator {
    pub fn new(short_period: u32, long_period: u32) -> ADOscillator {
        Self {
            short_history: FixedQueue::new(short_period),
            long_history: FixedQueue::new(long_period),
            short_ema: EMA::new(short_period),
            long_ema: EMA::new(long_period),
            long_ad_line: ADLine::new(),
            short_ad_line: ADLine::new(),
        }
    }
}

impl Indicator<[f64; 4], Option<f64>> for ADOscillator {
    fn next(&mut self, input: [f64; 4]) -> Option<f64> {
        let short_ad = self.short_ad_line.next(input);
        let ema_short = self.short_ema.next(short_ad);
        self.short_history.add(ema_short);

        let long_ad = self.long_ad_line.next(input);
        let ema_long = self.long_ema.next(long_ad);
        self.long_history.add(ema_long);
        println!("long {} short {}",ema_long,ema_short);

        return if self.long_history.is_full() {
            let long_index: i32 = (self.long_history.size() as i32 - 1 ) as i32;
            let short_index: i32 = (self.short_history.size() as i32 - 1 ) as i32;
            let res = self.long_history.at(long_index)
                .and_then(|lma| self.short_history.at(short_index).map(|sma| sma - lma));
            res
        } else {
            None
        };
    }

    fn reset(&mut self) {
        self.long_history.clear();
        self.short_history.clear();
    }
}


#[cfg(test)]
mod tests {
    use crate::{ADLine, ADOscillator};
    use ta_common::traits::Indicator;

    #[test]
    fn it_works() {
        let mut ad_line = ADLine::new();
        let ad = ad_line.next([200f64, 100f64, 170f64, 10f64]);
        assert_eq!(4_f64, ad);
        let ad = ad_line.next([200f64, 100f64, 170f64, 10f64]);
        assert_eq!(8_f64, ad);
        ad_line.reset();
        let ad = ad_line.next([200f64, 100f64, 170f64, 10f64]);
        assert_eq!(4_f64, ad);
    }

    #[test]
    fn adosc_works() {
        let mut ad_osc = ADOscillator::new(2, 5);
        assert_eq!(ad_osc.next([82.15, 81.29, 81.59, 5_653_100.00, ]), None);
        assert_eq!(ad_osc.next([81.89, 80.64, 81.06, 6_447_400.00, ]), None);
        assert_eq!(ad_osc.next([83.03, 81.31, 82.87, 7_690_900.00, ]), None);
        assert_eq!(ad_osc.next([83.30, 82.65, 83.00, 3_831_400.00, ]), None);
        assert_eq!(ad_osc.next([83.85, 83.07, 83.61, 4_455_100.00, ]), Some(1_900_759.84734648));
        assert_eq!(ad_osc.next([83.90, 83.11, 83.15, 3_798_000.00, ]), Some(399262.0395204937));
        assert_eq!(ad_osc.next([83.33, 82.49, 82.84, 3_936_200.00, ]), Some(-241806.81544536876));
        assert_eq!(ad_osc.next([84.30, 82.30, 83.99, 4_732_000.00, ]), Some(757828.2868834068));
        assert_eq!(ad_osc.next([84.84, 84.15, 84.55, 4_841_300.00, ]), Some(1068830.2845185758));
        assert_eq!(ad_osc.next([85.00, 84.11, 84.36, 3_915_300.00, ]), Some(328526.2457354958));
        assert_eq!(ad_osc.next([85.90, 84.03, 85.53, 6_830_800.00, ]), Some(1466909.2959969565));
        assert_eq!(ad_osc.next([86.58, 85.39, 86.54, 6_694_100.00, ]), Some(3475262.2871407317));
        assert_eq!(ad_osc.next([86.98, 85.76, 86.89, 5_293_600.00, ]), Some(4653474.793312617));
        assert_eq!(ad_osc.next([88.00, 87.17, 87.77, 7_985_800.00, ]), Some(5067839.26497877));
        assert_eq!(ad_osc.next([87.87, 87.01, 87.29, 4_807_900.00, ]), Some(3474675.6158188656));
    }
}
