# ADOscillator
```
use ta_common::traits::Indicator;
use ad_rs::ADOscillator;

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
```

