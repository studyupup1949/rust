pub(super) const SNAPSHOT_FIELDS: &[&str] = &[
    "schema",
    "snapshot_id",
    "engagement_id",
    "captured_at",
    "source_boundary",
    "sources",
    "records",
    "metadata",
];
pub(super) const SPACE_FIELDS: &[&str] = &[
    "schema",
    "space_id",
    "engagement_id",
    "snapshot_id",
    "package_id",
    "cells",
    "contexts",
    "incidences",
    "morphisms",
    "invariants",
    "policies",
    "metadata",
];
pub(super) const REPORT_FIELDS: &[&str] = &[
    "schema",
    "report_type",
    "report_version",
    "tool",
    "input",
    "result",
    "projection",
    "warnings",
];
pub(super) const PROJECTION_REQUEST_FIELDS: &[&str] = &[
    "schema",
    "projection_id",
    "space_id",
    "audience",
    "purpose",
    "include_ids",
    "exclude_ids",
    "policy_ids",
    "metadata",
];
pub(super) const CELL_FIELDS: &[&str] = &[
    "id",
    "cell_type",
    "title",
    "context_ids",
    "source_ids",
    "provenance",
    "metadata",
];
pub(super) const CONTEXT_FIELDS: &[&str] =
    &["id", "context_type", "title", "provenance", "metadata"];
pub(super) const INCIDENCE_FIELDS: &[&str] = &[
    "id",
    "relation_type",
    "from_id",
    "to_id",
    "context_ids",
    "evidence_ids",
    "strength",
    "provenance",
    "metadata",
];
pub(super) const REVIEW_EVENT_FIELDS: &[&str] = &[
    "schema",
    "review_event_id",
    "engagement_id",
    "target_ids",
    "outcome",
    "reviewer_id",
    "reviewed_at",
    "reason",
    "evidence_ids",
    "metadata",
];
pub(super) const HYPOTHESIS_EVENT_FIELDS: &[&str] = &[
    "schema",
    "hypothesis_event_id",
    "engagement_id",
    "target_hypothesis_id",
    "outcome",
    "reviewer_id",
    "reviewed_at",
    "reason",
    "evidence_ids",
    "metadata",
];
pub(super) const RELATION_FIELDS: &[&str] = &["relation_type", "from_record_id", "to_record_id"];
pub(super) const PROVENANCE_FIELDS: &[&str] = &["origin", "actor", "review_status"];
