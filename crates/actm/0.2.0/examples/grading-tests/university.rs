use std::collections::BTreeMap;

use actm::prelude::*;

// First we define the context type

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct UniversityContext {
    /// The students and their current grades
    pub students: BTreeMap<String, Vec<u32>>,
}

// Then the input type

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum UniversityInput {
    Get,
    Grade(String, u32),
}

// Then the output type

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct UniversityOutput(pub BTreeMap<String, Vec<u32>>);

wrapped_event!(UniversityOutputEvent, UniversityOutput);
wrapped_event!(UniveristyInputEvent, UniversityInput);

async fn university_event_handler(
    mut context: UniversityContext,
    event: UniveristyInputEvent,
) -> (UniversityContext, Option<UniversityOutputEvent>) {
    match event.into_inner() {
        UniversityInput::Grade(student, grade) => {
            let grades = context.students.entry(student).or_default();
            grades.push(grade);
            (context, None)
        }
        UniversityInput::Get => {
            let event = UniversityOutputEvent::from(UniversityOutput(context.students.clone()));
            (context, Some(event))
        }
    }
}

async_actor!(
    University,
    UniveristyInputEvent,
    UniversityOutputEvent,
    UniversityContext,
    university_event_handler
);
