-- Initial database schema for ADS-B Anomaly Detection System
-- Creates tables for aircraft observations, sessions, and anomaly detections

-- Aircraft observations table: stores normalized aircraft data from each poll
CREATE TABLE aircraft_observations (
    id INTEGER PRIMARY KEY,
    ts_ms INTEGER NOT NULL,
    hex TEXT NOT NULL,
    flight TEXT NULL,
    lat REAL NULL,
    lon REAL NULL,
    altitude INTEGER NULL,
    gs REAL NULL,
    rssi REAL NULL,
    msg_count_total INTEGER NULL,
    raw_json TEXT NOT NULL
);

-- Indexes for efficient time-series queries
CREATE INDEX idx_obs_ts_hex ON aircraft_observations(ts_ms, hex);
CREATE INDEX idx_obs_hex_ts ON aircraft_observations(hex, ts_ms);

-- Aircraft sessions table: maintains per-aircraft state and capabilities
CREATE TABLE aircraft_sessions (
    hex TEXT PRIMARY KEY,
    first_seen_ms INTEGER NOT NULL,
    last_seen_ms INTEGER NOT NULL,
    last_msg_total INTEGER NULL,
    message_count INTEGER NOT NULL DEFAULT 0,
    has_position INTEGER NOT NULL DEFAULT 0,
    has_altitude INTEGER NOT NULL DEFAULT 0,
    has_callsign INTEGER NOT NULL DEFAULT 0,
    flight TEXT NULL,
    lat REAL NULL,
    lon REAL NULL,
    altitude INTEGER NULL,
    speed REAL NULL,
    tier_temporal INTEGER NOT NULL DEFAULT 0,
    tier_signal INTEGER NOT NULL DEFAULT 0,
    tier_identity INTEGER NOT NULL DEFAULT 0,
    tier_behavioral INTEGER NOT NULL DEFAULT 0
);

-- Anomaly detections table: stores detected anomalies with confidence scores
CREATE TABLE anomaly_detections (
    id INTEGER PRIMARY KEY,
    ts_ms INTEGER NOT NULL,
    hex TEXT NOT NULL,
    anomaly_type TEXT NOT NULL,
    confidence REAL NOT NULL,
    details_json TEXT NULL,
    reviewed INTEGER NOT NULL DEFAULT 0
);

-- Index for efficient anomaly queries by type and time
CREATE INDEX idx_anomalies_type_ts ON anomaly_detections(anomaly_type, ts_ms);
CREATE INDEX idx_anomalies_hex_ts ON anomaly_detections(hex, ts_ms);
