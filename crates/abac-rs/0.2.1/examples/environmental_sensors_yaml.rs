//! Environmental sensor monitoring using YAML configuration.
//!
//! This example extends the basic environmental_sensors example by adding support
//! for loading sensor definitions and alert rules from YAML configuration files.
//!
//! Run with: `cargo run --example environmental_sensors_yaml -p abac-rs`

use abac_rs::{
    AbacPolicyLocal, AbacRequest, AbacRule, AttributeType, AttributeValue, Decision, Matcher,
};
use std::collections::HashMap;

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

fn load_config_from_yaml(yaml_str: &str) -> Result<SensorConfig, serde_yml::Error> {
    serde_yml::from_str(yaml_str)
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
    println!("=== Environmental Sensor Monitoring System (YAML Config) ===");
    println!();

    // Example YAML configuration
    let yaml_config = r#"
sensor_types:
  - name: greenhouse
    alerts:
      - name: greenhouse-heat
        description: "Temperature >= 40.0°C"
        sensor_group: greenhouse
        measurement: temperature
        threshold:
          type: float
          value: 40.0
        comparison: above
      - name: greenhouse-dry
        description: "Humidity <= 30.0%"
        sensor_group: greenhouse
        measurement: humidity
        threshold:
          type: float
          value: 30.0
        comparison: below

  - name: warehouse
    alerts:
      - name: warehouse-heat
        description: "Temperature >= 35.0°C"
        sensor_group: warehouse
        measurement: temperature
        threshold:
          type: float
          value: 35.0
        comparison: above

  - name: outdoor
    alerts:
      - name: outdoor-storm
        description: "Wind speed >= 80.0 km/h"
        sensor_group: outdoor
        measurement: wind_speed
        threshold:
          type: float
          value: 80.0
        comparison: above
      - name: outdoor-dark
        description: "Light level <= 10"
        sensor_group: outdoor
        measurement: light_level
        threshold:
          type: integer
          value: 10
        comparison: below
"#;

    let config = load_config_from_yaml(yaml_config).expect("Failed to parse YAML configuration");

    let mut alerts_by_type = build_alerts_from_config(&config);

    println!("Active alert conditions:");
    if let Some(alerts) = alerts_by_type.get("greenhouse") {
        print_alert_conditions("Greenhouse", alerts);
    }
    if let Some(alerts) = alerts_by_type.get("warehouse") {
        print_alert_conditions("Warehouse", alerts);
    }
    if let Some(alerts) = alerts_by_type.get("outdoor") {
        print_alert_conditions("Outdoor", alerts);
    }
    println!();

    // Reading 1: greenhouse heat alert (42.5°C exceeds 40.0°C threshold)
    if let Some(alerts) = alerts_by_type.get_mut("greenhouse") {
        evaluate_reading(
            "greenhouse-01",
            "greenhouse",
            &[
                ("temperature", AttributeType::Float(42.5)),
                ("humidity", AttributeType::Float(65.0)),
            ],
            alerts,
        );
    }

    // Reading 2: greenhouse dry alert (25% humidity below 30% threshold)
    if let Some(alerts) = alerts_by_type.get_mut("greenhouse") {
        evaluate_reading(
            "greenhouse-01",
            "greenhouse",
            &[
                ("temperature", AttributeType::Float(32.0)),
                ("humidity", AttributeType::Float(25.0)),
            ],
            alerts,
        );
    }

    // Reading 3: warehouse heat warning (36.5°C exceeds 35.0°C threshold)
    if let Some(alerts) = alerts_by_type.get_mut("warehouse") {
        evaluate_reading(
            "warehouse-A",
            "warehouse",
            &[("temperature", AttributeType::Float(36.5))],
            alerts,
        );
    }

    // Reading 4: outdoor storm warning (95 km/h exceeds 80 km/h threshold)
    if let Some(alerts) = alerts_by_type.get_mut("outdoor") {
        evaluate_reading(
            "rooftop-wx",
            "outdoor",
            &[
                ("wind_speed", AttributeType::Float(95.0)),
                ("light_level", AttributeType::Integer(45)),
            ],
            alerts,
        );
    }

    // Reading 5: normal greenhouse reading (no alerts triggered)
    if let Some(alerts) = alerts_by_type.get_mut("greenhouse") {
        evaluate_reading(
            "greenhouse-01",
            "greenhouse",
            &[
                ("temperature", AttributeType::Float(32.0)),
                ("humidity", AttributeType::Float(55.0)),
            ],
            alerts,
        );
    }

    println!("=== Configuration loaded from YAML ===");
    println!("Sensor types configured: {}", config.sensor_types.len());
    for sensor_type in &config.sensor_types {
        println!(
            "  - {}: {} alerts",
            sensor_type.name,
            sensor_type.alerts.len()
        );
    }
}
