use super::*;

pub(super) fn subject_seeds(
    space: &AdvisorySpaceEnvelope,
    obstructions: &[Value],
    hypotheses: &[Value],
    falsifiers: &[Value],
    candidates: &[Value],
    relations: &[RelationTriple],
) -> AdvisoryResult<BTreeMap<String, SubjectSeed>> {
    let mut seeds = BTreeMap::new();
    for cell in &space.cells {
        if let Some(id) = cell.get("id").and_then(Value::as_str) {
            seed(&mut seeds, id, cell_type(cell)).absorb_value(cell, cell_modality(cell));
        }
    }
    for obstruction in obstructions {
        if let Some(id) = obstruction.get("id").and_then(Value::as_str) {
            seed(&mut seeds, id, "obstruction").absorb_value(obstruction, None);
        }
    }
    for hypothesis in hypotheses {
        if let Some(id) = item_id(hypothesis, &["id", "hypothesis_id"]) {
            let modality =
                hypothesis_status(hypothesis).map(|status| format!("hypothesis:{status}"));
            seed(&mut seeds, id, "hypothesis").absorb_value(hypothesis, modality.as_deref());
        }
    }
    for falsifier in falsifiers {
        if let Some(id) = falsifier.get("id").and_then(Value::as_str) {
            seed(&mut seeds, id, "falsifier").absorb_value(falsifier, Some("falsifier"));
        }
    }
    for candidate in candidates {
        if let Some(id) = candidate.get("id").and_then(Value::as_str) {
            let seed = seed(&mut seeds, id, "completion_candidate");
            seed.absorb_value(candidate, None);
            for invariant in string_values(candidate, "affected_invariant_ids") {
                seed.invariant_states
                    .insert(invariant, InvariantSatisfaction::Satisfied);
            }
        }
    }
    for relation in relations {
        if let Some(seed) = seeds.get_mut(&relation.subject) {
            seed.relations.insert(relation.clone());
        }
        if let Some(seed) = seeds.get_mut(&relation.object) {
            seed.relations.insert(incoming_relation_triple(relation));
        }
    }
    Ok(seeds)
}

pub(super) fn relation_triples(
    space: &AdvisorySpaceEnvelope,
    candidates: &[Value],
    falsifiers: &[Value],
    argumentation_incidences: &[Value],
) -> AdvisoryResult<Vec<RelationTriple>> {
    let mut relations = Vec::new();
    for incidence in space.incidences.iter().chain(argumentation_incidences) {
        if let (Some(subject), Some(relation), Some(object)) = (
            incidence.get("from_id").and_then(Value::as_str),
            incidence.get("relation_type").and_then(Value::as_str),
            incidence.get("to_id").and_then(Value::as_str),
        ) {
            relations.push(relation_triple(subject, relation, object));
        }
    }
    for candidate in candidates {
        let Some(candidate_id) = candidate.get("id").and_then(Value::as_str) else {
            continue;
        };
        for obstruction_id in string_values(candidate, "resolves_obstruction_ids") {
            relations.push(relation_triple(candidate_id, "resolves", &obstruction_id));
        }
        for hypothesis_id in candidate
            .pointer("/metadata/derived_from_hypothesis_id")
            .and_then(Value::as_str)
            .into_iter()
            .chain(
                candidate
                    .get("supported_hypothesis_ids")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str),
            )
        {
            relations.push(relation_triple(candidate_id, "derives_from", hypothesis_id));
        }
    }
    for falsifier in falsifiers {
        if let (Some(falsifier_id), Some(hypothesis_id)) = (
            falsifier.get("id").and_then(Value::as_str),
            falsifier
                .pointer("/metadata/falsifies")
                .and_then(Value::as_str),
        ) {
            relations.push(relation_triple(falsifier_id, "falsifies", hypothesis_id));
        }
    }
    relations.sort();
    relations.dedup();
    Ok(relations)
}

pub(super) fn seed<'a>(
    seeds: &'a mut BTreeMap<String, SubjectSeed>,
    id: &str,
    role: &str,
) -> &'a mut SubjectSeed {
    seeds
        .entry(id.to_owned())
        .or_insert_with(|| SubjectSeed::new(id, role))
}

pub(super) fn relation_triple(subject: &str, relation: &str, object: &str) -> RelationTriple {
    RelationTriple {
        subject: subject.to_owned(),
        relation: relation.to_owned(),
        object: object.to_owned(),
    }
}

pub(super) fn incoming_relation_triple(relation: &RelationTriple) -> RelationTriple {
    relation_triple(
        &relation.object,
        &format!("incoming:{}", relation.relation),
        &relation.subject,
    )
}
