use serde_json::{Value, json};

use crate::error::{Error, Result};

/// Mirrors `sensor_msgs/JointState` using the JSON envelope shared with the
/// Python SDK.
#[derive(Debug, Clone, PartialEq)]
pub struct JointState {
    pub names: Vec<String>,
    pub positions: Vec<f64>,
    pub velocity: Vec<f64>,
    pub effort: Vec<f64>,
    pub stamp: f64,
    pub frame_id: String,
}

impl Default for JointState {
    fn default() -> Self {
        Self {
            names: Vec::new(),
            positions: Vec::new(),
            velocity: Vec::new(),
            effort: Vec::new(),
            stamp: 0.0,
            frame_id: String::new(),
        }
    }
}

impl JointState {
    pub fn to_json(&self) -> Result<Vec<u8>> {
        let value = json!({
            "type": "JointState",
            "stamp": stamp_or_now(self.stamp),
            "frame_id": self.frame_id,
            "names": self.names,
            "positions": self.positions,
            "velocity": self.velocity,
            "effort": self.effort,
        });
        encode(value)
    }
}

/// Mirrors `sensor_msgs/Joy`.
#[derive(Debug, Clone, PartialEq)]
pub struct Joy {
    pub axes: Vec<f64>,
    pub buttons: Vec<i32>,
    pub stamp: f64,
}

impl Default for Joy {
    fn default() -> Self {
        Self {
            axes: Vec::new(),
            buttons: Vec::new(),
            stamp: 0.0,
        }
    }
}

impl Joy {
    pub fn to_json(&self) -> Result<Vec<u8>> {
        let value = json!({
            "type": "Joy",
            "stamp": stamp_or_now(self.stamp),
            "axes": self.axes,
            "buttons": self.buttons,
        });
        encode(value)
    }
}

/// Joystick command with sequence ID for packet ordering.
#[derive(Debug, Clone, PartialEq)]
pub struct JoystickCommand {
    pub sequence_id: u64,
    pub axes: Vec<f64>,
    pub buttons: Vec<i32>,
    pub stamp: f64,
}

impl Default for JoystickCommand {
    fn default() -> Self {
        Self {
            sequence_id: 0,
            axes: Vec::new(),
            buttons: Vec::new(),
            stamp: 0.0,
        }
    }
}

impl JoystickCommand {
    pub fn to_json(&self) -> Result<Vec<u8>> {
        let value = json!({
            "type": "JoystickCommand",
            "sequence_id": self.sequence_id,
            "stamp": stamp_or_now(self.stamp),
            "axes": self.axes,
            "buttons": self.buttons,
        });
        encode(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ControlMessage {
    JointState(JointState),
    Joy(Joy),
    JoystickCommand(JoystickCommand),
    Raw(Value),
}

/// Decode a control message from JSON bytes.
///
/// Known `"type"` envelopes are returned as typed Rust structs. Unknown JSON
/// is returned as [`ControlMessage::Raw`], matching Python's raw-dict fallback.
pub fn decode_control(payload: &[u8]) -> Result<ControlMessage> {
    let value: Value = serde_json::from_slice(payload)
        .map_err(|e| Error::Ffi(format!("control JSON decode failed: {e}")))?;
    let msg_type = value.get("type").and_then(Value::as_str);
    match msg_type {
        Some("JointState") => Ok(ControlMessage::JointState(JointState {
            names: string_vec(value.get("names")),
            positions: f64_vec(value.get("positions")),
            velocity: f64_vec(value.get("velocity")),
            effort: f64_vec(value.get("effort")),
            stamp: f64_field(&value, "stamp"),
            frame_id: string_field(&value, "frame_id"),
        })),
        Some("Joy") => Ok(ControlMessage::Joy(Joy {
            axes: f64_vec(value.get("axes")),
            buttons: i32_vec(value.get("buttons")),
            stamp: f64_field(&value, "stamp"),
        })),
        Some("JoystickCommand") => Ok(ControlMessage::JoystickCommand(JoystickCommand {
            sequence_id: value
                .get("sequence_id")
                .and_then(Value::as_u64)
                .unwrap_or(0),
            axes: f64_vec(value.get("axes")),
            buttons: i32_vec(value.get("buttons")),
            stamp: f64_field(&value, "stamp"),
        })),
        _ => Ok(ControlMessage::Raw(value)),
    }
}

fn encode(value: Value) -> Result<Vec<u8>> {
    serde_json::to_vec(&value)
        .map_err(|e| Error::Ffi(format!("control JSON encode failed: {e}")))
}

fn stamp_or_now(stamp: f64) -> f64 {
    if stamp != 0.0 {
        return stamp;
    }
    crate::fabric_now_us() as f64 / 1_000_000.0
}

fn string_field(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

fn f64_field(value: &Value, key: &str) -> f64 {
    value.get(key).and_then(Value::as_f64).unwrap_or(0.0)
}

fn string_vec(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn f64_vec(value: Option<&Value>) -> Vec<f64> {
    value
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(Value::as_f64).collect())
        .unwrap_or_default()
}

fn i32_vec(value: Option<&Value>) -> Vec<i32> {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_i64)
                .filter_map(|n| i32::try_from(n).ok())
                .collect()
        })
        .unwrap_or_default()
}
