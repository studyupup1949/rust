//! Environmental sensor monitoring using ABAC rules for boundary-condition alerts.
//!
//! Demonstrates custom matchers, multiple attribute types (Float, Integer, String),
//! sensor-type grouping, and the builder API.
//!
//! Run with: `cargo run --example environmental_sensors -p abac-rs`
//!
//! ## YAML Configuration
//!
//! For a version that loads sensor definitions from YAML files, see:
//! - `environmental_sensors_yaml.rs` - Inline YAML configuration
//! - `environmental_sensors_yaml_file.rs` - Load from external YAML file
//! - `SENSOR_CONFIG_README.md` - YAML format documentation

use abac_rs::{
    AbacPolicyLocal, AbacRequest, AbacRule, AttributeType, AttributeValue, Decision, Matcher,
};

fn to_f64(attr: &AttributeType) -> Option<f64> {
    match attr {
        AttributeType::Float(v) => Some(*v),
        AttributeType::Integer(v) => Some(*v as f64),
        _ => None,
    }
}

struct ThresholdAboveMatcher;

impl Matcher for ThresholdAboveMatcher {
    fn matches(
        &self,
        rule_value: &AttributeValue,
        request_value: &AttributeType,
        _request_groups: &[AttributeType],
    ) -> bool {
        match rule_value {
            AttributeValue::All => true,
            AttributeValue::Specific(thresholds) => {
                let Some(actual) = to_f64(request_value) else {
                    return false;
                };
                thresholds
                    .iter()
                    .any(|t| to_f64(t).is_some_and(|limit| actual >= limit))
            }
        }
    }

    fn supports_bloom_filter(&self) -> bool {
        false
    }
}

struct ThresholdBelowMatcher;

impl Matcher for ThresholdBelowMatcher {
    fn matches(
        &self,
        rule_value: &AttributeValue,
        request_value: &AttributeType,
        _request_groups: &[AttributeType],
    ) -> bool {
        match rule_value {
            AttributeValue::All => true,
            AttributeValue::Specific(thresholds) => {
                let Some(actual) = to_f64(request_value) else {
                    return false;
                };
                thresholds
                    .iter()
                    .any(|t| to_f64(t).is_some_and(|limit| actual <= limit))
            }
        }
    }

    fn supports_bloom_filter(&self) -> bool {
        false
    }
}

struct AlertCheck {
    name: String,
    description: String,
    policy: AbacPolicyLocal,
}

fn make_alert(
    name: &str,
    description: &str,
    sensor_group: &str,
    measurement_dim: &str,
    threshold: AttributeType,
    matcher: Box<dyn Matcher>,
) -> AlertCheck {
    let rule = AbacRule::builder(name)
        .dimension_values(
            "sensor_type",
            vec![AttributeType::String(format!("group:{sensor_group}"))],
        )
        .dimension_values(measurement_dim, vec![threshold])
        .enabled(true)
        .build();

    let mut policy = AbacPolicyLocal::new();
    policy.register_matcher(measurement_dim, matcher);
    policy.add_rule(rule).unwrap();

    AlertCheck {
        name: name.to_string(),
        description: description.to_string(),
        policy,
    }
}

fn build_alerts_for(sensor_type: &str) -> Vec<AlertCheck> {
    match sensor_type {
        "greenhouse" => vec![
            make_alert(
                "greenhouse-heat",
                "Temperature >= 40.0\u{00b0}C",
                "greenhouse",
                "temperature",
                AttributeType::Float(40.0),
                Box::new(ThresholdAboveMatcher),
            ),
            make_alert(
                "greenhouse-dry",
                "Humidity <= 30.0%",
                "greenhouse",
                "humidity",
                AttributeType::Float(30.0),
                Box::new(ThresholdBelowMatcher),
            ),
        ],
        "warehouse" => vec![make_alert(
            "warehouse-heat",
            "Temperature >= 35.0\u{00b0}C",
            "warehouse",
            "temperature",
            AttributeType::Float(35.0),
            Box::new(ThresholdAboveMatcher),
        )],
        "outdoor" => vec![
            make_alert(
                "outdoor-storm",
                "Wind speed >= 80.0 km/h",
                "outdoor",
                "wind_speed",
                AttributeType::Float(80.0),
                Box::new(ThresholdAboveMatcher),
            ),
            make_alert(
                "outdoor-dark",
                "Light level <= 10",
                "outdoor",
                "light_level",
                AttributeType::Integer(10),
                Box::new(ThresholdBelowMatcher),
            ),
        ],
        _ => vec![],
    }
}

fn evaluate_reading(
    sensor_id: &str,
    sensor_type: &str,
    measurements: &[(&str, AttributeType)],
    alerts: &mut [AlertCheck],
) {
    println!("--- Sensor: {sensor_id} ({sensor_type}) ---");

    let reading: Vec<String> = measurements
        .iter()
        .map(|(dim, val)| match val {
            AttributeType::Float(v) => format!("{dim}={v}"),
            AttributeType::Integer(v) => format!("{dim}={v}"),
            _ => format!("{dim}={val:?}"),
        })
        .collect();
    println!("  Readings: {}", reading.join(", "));

    let mut any_alert = false;
    for alert in alerts.iter_mut() {
        let mut request = AbacRequest::new();
        request
            .add_attribute(
                "sensor_type",
                AttributeType::String(sensor_id.to_string()),
                vec![AttributeType::String(format!("group:{sensor_type}"))],
            )
            .unwrap();

        for (dim, val) in measurements {
            request.add_attribute(*dim, val.clone(), vec![]).unwrap();
        }

        if alert.policy.evaluate(&request) == Decision::Allow {
            println!("  ALERT [{}]: {}", alert.name, alert.description);
            any_alert = true;
        }
    }

    if !any_alert {
        println!("  Status: Normal — no alerts triggered");
    }
    println!();
}

fn print_alert_conditions(label: &str, alerts: &[AlertCheck]) {
    println!("  {label}:");
    for alert in alerts {
        println!("    - [{}] {}", alert.name, alert.description);
    }
}

fn main() {
    println!("=== Environmental Sensor Monitoring System ===");
    println!();

    let mut greenhouse_alerts = build_alerts_for("greenhouse");
    let mut warehouse_alerts = build_alerts_for("warehouse");
    let mut outdoor_alerts = build_alerts_for("outdoor");

    println!("Active alert conditions:");
    print_alert_conditions("Greenhouse", &greenhouse_alerts);
    print_alert_conditions("Warehouse", &warehouse_alerts);
    print_alert_conditions("Outdoor", &outdoor_alerts);
    println!();

    // Reading 1: greenhouse heat alert (42.5°C exceeds 40.0°C threshold)
    evaluate_reading(
        "greenhouse-01",
        "greenhouse",
        &[
            ("temperature", AttributeType::Float(42.5)),
            ("humidity", AttributeType::Float(65.0)),
        ],
        &mut greenhouse_alerts,
    );

    // Reading 2: greenhouse dry alert (25% humidity below 30% threshold)
    evaluate_reading(
        "greenhouse-01",
        "greenhouse",
        &[
            ("temperature", AttributeType::Float(32.0)),
            ("humidity", AttributeType::Float(25.0)),
        ],
        &mut greenhouse_alerts,
    );

    // Reading 3: warehouse heat warning (36.5°C exceeds 35.0°C threshold)
    evaluate_reading(
        "warehouse-A",
        "warehouse",
        &[("temperature", AttributeType::Float(36.5))],
        &mut warehouse_alerts,
    );

    // Reading 4: outdoor storm warning (95 km/h exceeds 80 km/h threshold)
    evaluate_reading(
        "rooftop-wx",
        "outdoor",
        &[
            ("wind_speed", AttributeType::Float(95.0)),
            ("light_level", AttributeType::Integer(45)),
        ],
        &mut outdoor_alerts,
    );

    // Reading 5: normal greenhouse reading (no alerts triggered)
    evaluate_reading(
        "greenhouse-01",
        "greenhouse",
        &[
            ("temperature", AttributeType::Float(32.0)),
            ("humidity", AttributeType::Float(55.0)),
        ],
        &mut greenhouse_alerts,
    );
}
