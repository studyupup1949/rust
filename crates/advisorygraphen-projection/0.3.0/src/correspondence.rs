use advisorygraphen_core::{AdvisoryError, AdvisoryResult, AdvisorySpaceEnvelope};
use higher_graphen_core::{Id, ParticipantRef};
use higher_graphen_projection::{project_correspondence, ProjectionAudience, ProjectionPurpose};
use higher_graphen_reasoning::correspondence::{
    derive_correspondence_candidates, CorrespondenceDetectionInput, CorrespondenceScope,
    CorrespondenceSubject, InvariantSatisfaction, InvariantState, TypedRelation,
};
use higher_graphen_reasoning::gluing::attempt_gluing;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

const MAX_REVIEW_FOCUS_CORRESPONDENCES: usize = 24;

pub fn correspondence_analysis(
    space: &AdvisorySpaceEnvelope,
    obstructions: &[Value],
    hypotheses: &[Value],
    falsifiers: &[Value],
    candidates: &[Value],
    argumentation_incidences: &[Value],
) -> AdvisoryResult<Value> {
    let relations = relation_triples(space, candidates, falsifiers, argumentation_incidences)?;
    let subjects = subject_seeds(
        space,
        obstructions,
        hypotheses,
        falsifiers,
        candidates,
        &relations,
    )?
    .into_values()
    .map(SubjectSeed::into_subject)
    .collect::<AdvisoryResult<Vec<_>>>()?;
    let context = id("context:advisory-correspondence-analysis")?;
    let provenance = id("source:advisorygraphen-correspondence-analysis")?;
    let subject_count = subjects.len();
    let result = derive_correspondence_candidates(
        CorrespondenceDetectionInput::new(context, provenance, subjects)
            .with_scope(CorrespondenceScope::All),
    )
    .map_err(hg_err)?;

    let mut ranked = Vec::new();
    let mut failure_count = 0_u64;
    let mut review_count = 0_u64;
    let mut success_count = 0_u64;

    for mut candidate in result.candidates {
        let gluing = attempt_gluing(&candidate).map_err(hg_err)?;
        let gluing_kind = gluing.result.kind().to_owned();
        match gluing_kind.as_str() {
            "failure" => failure_count += 1,
            "candidate" => review_count += 1,
            "success" => success_count += 1,
            _ => {}
        }
        let witness_kinds = unique_sorted(
            candidate
                .overlap_witnesses
                .iter()
                .map(|witness| witness.witness_kind.kind().to_owned()),
        );
        let difference_severities = unique_sorted(
            candidate
                .difference_witnesses
                .iter()
                .map(|witness| witness.severity.kind().to_owned()),
        );
        candidate.gluing = Some(gluing);
        let projection = serde_json::to_value(project_correspondence(
            &candidate,
            ProjectionAudience::AiAgent,
            ProjectionPurpose::Review,
        ))?;
        let candidate_value = serde_json::to_value(candidate)?;
        let (selection_score, selection_reasons) = selection_score(
            &candidate_value,
            &gluing_kind,
            &witness_kinds,
            &difference_severities,
        );
        ranked.push(RankedCorrespondence {
            candidate: candidate_value,
            projection,
            gluing_kind,
            witness_kinds,
            difference_severities,
            selection_score,
            selection_reasons,
        });
    }
    let total_candidate_count = ranked.len();
    ranked.sort_by(|left, right| {
        right
            .selection_score
            .cmp(&left.selection_score)
            .then_with(|| left.id().cmp(&right.id()))
    });
    let selected = ranked
        .into_iter()
        .take(MAX_REVIEW_FOCUS_CORRESPONDENCES)
        .collect::<Vec<_>>();
    let selected_count = selected.len();
    let omitted_count = total_candidate_count.saturating_sub(selected_count);
    let review_focus_summaries = selected
        .iter()
        .enumerate()
        .map(|(index, candidate)| candidate.summary(index + 1))
        .collect::<Vec<_>>();
    let candidate_values = selected
        .iter()
        .map(|candidate| candidate.candidate.clone())
        .collect::<Vec<_>>();
    let projections = selected
        .iter()
        .map(|candidate| candidate.projection.clone())
        .collect::<Vec<_>>();

    Ok(json!({
        "schema": "advisorygraphen.correspondence_analysis.v1",
        "source": "highergraphen_0_5_correspondence_overlap_gluing",
        "subject_count": subject_count,
        "candidate_count": total_candidate_count,
        "emitted_candidate_count": selected_count,
        "omitted_candidate_count": omitted_count,
        "max_emitted_candidates": MAX_REVIEW_FOCUS_CORRESPONDENCES,
        "selection_policy": [
            "Always rank gluing failures first.",
            "Then rank gluing review candidates and blocking differences.",
            "Then rank major differences and non-surface structural witnesses.",
            "Omit low-signal success-only surface or evidence overlaps when the candidate set is large."
        ],
        "gluing_summary": {
            "failure": failure_count,
            "review_candidate": review_count,
            "success": success_count,
            "rule": "Gluing failures and review candidates are structural review prompts, not accepted advisory facts."
        },
        "review_focus_candidates": review_focus_summaries,
        "candidates": candidate_values,
        "ai_agent_projections": projections,
        "agent_rule": "Use review_focus_candidates first. Do not inspect omitted success-only correspondence candidates unless a reviewer asks for full trace expansion."
    }))
}
struct RankedCorrespondence {
    candidate: Value,
    projection: Value,
    gluing_kind: String,
    witness_kinds: Vec<String>,
    difference_severities: Vec<String>,
    selection_score: i64,
    selection_reasons: Vec<String>,
}
impl RankedCorrespondence {
    fn id(&self) -> String {
        self.candidate
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned()
    }

    fn summary(&self, rank: usize) -> Value {
        json!({
            "rank": rank,
            "candidate_id": self.candidate.get("id"),
            "selection_score": self.selection_score,
            "selection_reasons": self.selection_reasons,
            "gluing_kind": self.gluing_kind,
            "participants": participant_ids(&self.candidate),
            "overlap_witness_kinds": &self.witness_kinds,
            "difference_severities": &self.difference_severities,
            "review_status": self.candidate.get("reviewStatus")
        })
    }
}
#[derive(Clone)]
struct SubjectSeed {
    id: String,
    role: String,
    label: Option<String>,
    modality: Option<String>,
    contexts: BTreeSet<String>,
    evidence: BTreeSet<String>,
    invariants: BTreeSet<String>,
    invariant_states: BTreeMap<String, InvariantSatisfaction>,
    relations: BTreeSet<RelationTriple>,
}

impl SubjectSeed {
    fn new(id: &str, role: &str) -> Self {
        Self {
            id: id.to_owned(),
            role: role.to_owned(),
            label: None,
            modality: None,
            contexts: BTreeSet::new(),
            evidence: BTreeSet::new(),
            invariants: BTreeSet::new(),
            invariant_states: BTreeMap::new(),
            relations: BTreeSet::new(),
        }
    }

    fn absorb_value(&mut self, value: &Value, modality: Option<&str>) {
        self.label = self.label.clone().or_else(|| normalized_label(value));
        self.modality = self
            .modality
            .clone()
            .or_else(|| modality.map(str::to_owned));
        extend_strings(
            &mut self.contexts,
            value,
            &["context_ids", "location_context_ids"],
        );
        extend_strings(&mut self.evidence, value, &["source_ids", "evidence_ids"]);
        extend_strings(
            &mut self.invariants,
            value,
            &["affected_invariant_ids", "invariant_ids"],
        );
        for field in ["violated_invariant_id", "invariant_id"] {
            if let Some(invariant) = value.get(field).and_then(Value::as_str) {
                self.invariants.insert(invariant.to_owned());
                self.invariant_states
                    .insert(invariant.to_owned(), InvariantSatisfaction::Failed);
            }
        }
    }

    fn into_subject(self) -> AdvisoryResult<CorrespondenceSubject> {
        Ok(CorrespondenceSubject {
            participant: ParticipantRef::from_compact_id(id(&self.id)?),
            role: Some(self.role),
            normalized_label: self.label,
            modality: self.modality,
            contexts: ids(self.contexts)?,
            evidence: ids(self.evidence)?,
            invariants: ids(self.invariants)?,
            invariant_states: self
                .invariant_states
                .into_iter()
                .map(|(invariant, satisfaction)| {
                    Ok(InvariantState::new(id(&invariant)?, satisfaction))
                })
                .collect::<AdvisoryResult<Vec<_>>>()?,
            typed_relations: self
                .relations
                .into_iter()
                .map(|relation| {
                    TypedRelation::new(relation.subject, relation.relation, relation.object)
                        .map_err(hg_err)
                })
                .collect::<AdvisoryResult<Vec<_>>>()?,
        })
    }
}

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
struct RelationTriple {
    subject: String,
    relation: String,
    object: String,
}

mod extraction;
mod scoring;
mod util;

use extraction::*;
use scoring::*;
use util::*;
