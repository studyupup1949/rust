//! Environmental sensor monitoring using YAML configuration from file.
//!
//! This example demonstrates loading sensor definitions and alert rules from
//! an external YAML configuration file.
//!
//! Run with: `cargo run --example environmental_sensors_yaml_file -p abac-rs`

use abac_rs::{
    AbacPolicyLocal, AbacRequest, AbacRule, AttributeType, AttributeValue, Decision, Matcher,
};
use std::collections::HashMap;
use std::fs;

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

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum ValueType {
    Float { value: f64 },
    Integer { value: i64 },
    String { value: String },
}

impl ValueType {
    fn to_attribute_type(&self) -> AttributeType {
        match self {
            ValueType::Float { value } => AttributeType::Float(*value),
            ValueType::Integer { value } => AttributeType::Integer(*value),
            ValueType::String { value } => AttributeType::String(value.clone()),
        }
    }
}

#[derive(Debug, serde::Deserialize)]
struct AlertCondition {
    name: String,
    description: String,
    sensor_group: String,
    measurement: String,
    threshold: ValueType,
    #[serde(rename = "comparison")]
    comparison_type: String,
}

#[derive(Debug, serde::Deserialize)]
struct SensorType {
    name: String,
    alerts: Vec<AlertCondition>,
}

#[derive(Debug, serde::Deserialize)]
struct SensorConfig {
    sensor_types: Vec<SensorType>,
}

struct AlertCheck {
    name: String,
    description: String,
    policy: AbacPolicyLocal,
}

fn create_matcher(comparison_type: &str) -> Box<dyn Matcher> {
    match comparison_type {
        "above" | ">=" => Box::new(ThresholdAboveMatcher),
        "below" | "<=" => Box::new(ThresholdBelowMatcher),
        _ => panic!("Unknown comparison type: {}", comparison_type),
    }
}

fn make_alert_from_config(condition: &AlertCondition) -> AlertCheck {
    let rule = AbacRule::builder(&condition.name)
        .dimension_values(
            "sensor_type",
            vec![AttributeType::String(format!(
                "group:{}",
                condition.sensor_group
            ))],
        )
        .dimension_values(
            &condition.measurement,
            vec![condition.threshold.to_attribute_type()],
        )
        .enabled(true)
        .build();

    let mut policy = AbacPolicyLocal::new();
    policy.register_matcher(
        &condition.measurement,
        create_matcher(&condition.comparison_type),
    );
    policy.add_rule(rule).unwrap();

    AlertCheck {
        name: condition.name.clone(),
        description: condition.description.clone(),
        policy,
    }
}

fn load_config_from_file(path: &str) -> Result<SensorConfig, Box<dyn std::error::Error>> {
    let yaml_str = fs::read_to_string(path)?;
    let config = serde_yaml::from_str(&yaml_str)?;
    Ok(config)
}

fn build_alerts_from_config(config: &SensorConfig) -> HashMap<String, Vec<AlertCheck>> {
    let mut alerts_by_type = HashMap::new();

    for sensor_type in &config.sensor_types {
        let alerts: Vec<AlertCheck> = sensor_type
            .alerts
            .iter()
            .map(make_alert_from_config)
            .collect();
        alerts_by_type.insert(sensor_type.name.clone(), alerts);
    }

    alerts_by_type
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
    println!("=== Environmental Sensor Monitoring System (YAML File Config) ===");
    println!();

    // Load configuration from file
    let config_path = "crates/abac-rs/examples/sensor_config.yaml";
    let config = match load_config_from_file(config_path) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Error loading configuration from {}: {}", config_path, e);
            eprintln!("Make sure to run this example from the repository root directory.");
            return;
        }
    };

    println!("Configuration loaded from: {}", config_path);
    println!("Sensor types configured: {}", config.sensor_types.len());
    println!();

    let mut alerts_by_type = build_alerts_from_config(&config);

    println!("Active alert conditions:");
    for sensor_type in &config.sensor_types {
        if let Some(alerts) = alerts_by_type.get(&sensor_type.name) {
            print_alert_conditions(&sensor_type.name, alerts);
        }
    }
    println!();

    // Test scenarios for each sensor type

    // Greenhouse scenarios
    if let Some(alerts) = alerts_by_type.get_mut("greenhouse") {
        println!("=== Testing Greenhouse Sensors ===");
        evaluate_reading(
            "greenhouse-01",
            "greenhouse",
            &[
                ("temperature", AttributeType::Float(42.5)),
                ("humidity", AttributeType::Float(65.0)),
            ],
            alerts,
        );

        evaluate_reading(
            "greenhouse-02",
            "greenhouse",
            &[
                ("temperature", AttributeType::Float(32.0)),
                ("humidity", AttributeType::Float(25.0)),
            ],
            alerts,
        );

        evaluate_reading(
            "greenhouse-03",
            "greenhouse",
            &[
                ("temperature", AttributeType::Float(3.0)),
                ("humidity", AttributeType::Float(50.0)),
            ],
            alerts,
        );
    }

    // Warehouse scenarios
    if let Some(alerts) = alerts_by_type.get_mut("warehouse") {
        println!("=== Testing Warehouse Sensors ===");
        evaluate_reading(
            "warehouse-A",
            "warehouse",
            &[("temperature", AttributeType::Float(36.5))],
            alerts,
        );

        evaluate_reading(
            "warehouse-B",
            "warehouse",
            &[("temperature", AttributeType::Float(-2.0))],
            alerts,
        );
    }

    // Outdoor scenarios
    if let Some(alerts) = alerts_by_type.get_mut("outdoor") {
        println!("=== Testing Outdoor Sensors ===");
        evaluate_reading(
            "rooftop-wx",
            "outdoor",
            &[
                ("wind_speed", AttributeType::Float(95.0)),
                ("light_level", AttributeType::Integer(45)),
                ("temperature", AttributeType::Float(48.0)),
            ],
            alerts,
        );

        evaluate_reading(
            "parking-lot",
            "outdoor",
            &[
                ("wind_speed", AttributeType::Float(15.0)),
                ("light_level", AttributeType::Integer(5)),
                ("temperature", AttributeType::Float(22.0)),
            ],
            alerts,
        );
    }

    // Data center scenarios (if configured)
    if let Some(alerts) = alerts_by_type.get_mut("datacenter") {
        println!("=== Testing Data Center Sensors ===");
        evaluate_reading(
            "dc-rack-12",
            "datacenter",
            &[
                ("temperature", AttributeType::Float(29.5)),
                ("humidity", AttributeType::Float(65.0)),
            ],
            alerts,
        );

        evaluate_reading(
            "dc-rack-13",
            "datacenter",
            &[
                ("temperature", AttributeType::Float(25.0)),
                ("humidity", AttributeType::Float(15.0)),
            ],
            alerts,
        );
    }
}
