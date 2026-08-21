# Environmental Sensors Configuration (YAML/TOML)

This document describes the YAML and TOML configuration formats for the environmental sensors monitoring example in abac-rs.

## Overview

The YAML configuration allows you to define sensor types, their data fields, and alert conditions without modifying code. This makes it easy to:

- Add new sensor types
- Configure threshold values
- Define custom alert conditions
- Support different comparison operators

## Configuration Formats

Both YAML and TOML formats are supported and use the same structure. The examples below show both formats.

## YAML Schema

### Top-level Structure

```yaml
sensor_types:
  - name: <sensor-type-name>
    alerts:
      - <alert-definition>
      - <alert-definition>
      ...
```

### Alert Definition

Each alert has the following fields:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Unique identifier for the alert rule |
| `description` | string | Yes | Human-readable description of the alert condition |
| `sensor_group` | string | Yes | The sensor group this alert applies to (must match sensor type name) |
| `measurement` | string | Yes | The measurement dimension to check (e.g., "temperature", "humidity") |
| `threshold` | object | Yes | The threshold value (see below) |
| `comparison` | string | Yes | Comparison operator: "above" (>=) or "below" (<=) |

### Threshold Object

The threshold defines the value and type to compare against:

```yaml
threshold:
  type: <value-type>
  value: <value>
```

Supported types:
- `float`: Floating-point number (e.g., 40.0)
- `integer`: Whole number (e.g., 10)
- `string`: Text value (for future extensions)

## Example Configuration

### YAML Format

```yaml
sensor_types:
  # Greenhouse sensors
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

  # Data center sensors
  - name: datacenter
    alerts:
      - name: datacenter-overheat
        description: "Temperature >= 28.0°C"
        sensor_group: datacenter
        measurement: temperature
        threshold:
          type: float
          value: 28.0
        comparison: above
```

### TOML Format

```toml
[[sensor_types]]
name = "greenhouse"

[[sensor_types.alerts]]
name = "greenhouse-heat"
description = "Temperature >= 40.0°C"
sensor_group = "greenhouse"
measurement = "temperature"
comparison = "above"

[sensor_types.alerts.threshold]
type = "float"
value = 40.0

[[sensor_types.alerts]]
name = "greenhouse-dry"
description = "Humidity <= 30.0%"
sensor_group = "greenhouse"
measurement = "humidity"
comparison = "below"

[sensor_types.alerts.threshold]
type = "float"
value = 30.0

[[sensor_types]]
name = "datacenter"

[[sensor_types.alerts]]
name = "datacenter-overheat"
description = "Temperature >= 28.0°C"
sensor_group = "datacenter"
measurement = "temperature"
comparison = "above"

[sensor_types.alerts.threshold]
type = "float"
value = 28.0
```

## Common Use Cases

### Temperature Monitoring

Monitor high and low temperature thresholds:

```yaml
- name: freezer-warm
  description: "Freezer temperature above -15°C"
  sensor_group: freezer
  measurement: temperature
  threshold:
    type: float
    value: -15.0
  comparison: above

- name: freezer-critical
  description: "Freezer temperature above 0°C"
  sensor_group: freezer
  measurement: temperature
  threshold:
    type: float
    value: 0.0
  comparison: above
```

### Environmental Conditions

Monitor humidity, light levels, air quality:

```yaml
- name: room-humid
  description: "Humidity above 70%"
  sensor_group: office
  measurement: humidity
  threshold:
    type: float
    value: 70.0
  comparison: above

- name: air-quality-poor
  description: "CO2 level above 1000 ppm"
  sensor_group: office
  measurement: co2_ppm
  threshold:
    type: integer
    value: 1000
  comparison: above
```

### Weather Alerts

Monitor outdoor conditions:

```yaml
- name: high-wind
  description: "Wind speed above 60 km/h"
  sensor_group: outdoor
  measurement: wind_speed
  threshold:
    type: float
    value: 60.0
  comparison: above

- name: low-visibility
  description: "Visibility below 1000m"
  sensor_group: outdoor
  measurement: visibility_m
  threshold:
    type: integer
    value: 1000
  comparison: below
```

## Running the Examples

### Inline YAML Configuration

The `environmental_sensors_yaml` example includes YAML configuration embedded in the code:

```bash
cargo run --example environmental_sensors_yaml -p abac-rs
```

### External YAML File

The `environmental_sensors_yaml_file` example loads configuration from `sensor_config.yaml`:

```bash
# Run from the repository root
cargo run --example environmental_sensors_yaml_file -p abac-rs
```

### External TOML File

The `environmental_sensors_toml` example loads configuration from `sensor_config.toml`:

```bash
# Run from the repository root
cargo run --example environmental_sensors_toml -p abac-rs
```

Both YAML and TOML formats use the same data structure and are functionally equivalent. Choose the format you prefer.

## Extending the Configuration

### Adding Custom Measurement Types

To add support for new measurement dimensions:

1. Add the measurement field to your YAML configuration
2. Ensure your sensor readings include this dimension
3. The ABAC engine will automatically handle the new dimension

Example:
```yaml
- name: pressure-high
  description: "Atmospheric pressure above 1030 hPa"
  sensor_group: weather
  measurement: pressure_hpa
  threshold:
    type: float
    value: 1030.0
  comparison: above
```

### Custom Comparison Operators

The current implementation supports:
- `above` or `>=`: Triggers when value >= threshold
- `below` or `<=`: Triggers when value <= threshold

To add more operators (e.g., exact match, range), extend the `create_matcher()` function and implement a new `Matcher`.

## Integration with ABAC

The YAML configuration is translated into ABAC rules:

1. Each alert becomes an `AbacRule`
2. The `sensor_group` becomes a dimension with group matching
3. The `measurement` and `threshold` become dimension constraints
4. Custom matchers (`ThresholdAboveMatcher`, `ThresholdBelowMatcher`) implement the comparison logic

This demonstrates how ABAC can be used for:
- Real-time event filtering
- Boundary condition detection
- Multi-dimensional attribute matching
- Dynamic rule configuration
