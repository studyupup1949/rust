use std::collections::HashMap;

use actm::prelude::*;

use crate::Answers;

// First we define the context type, defining our internal state, for the professor actor

pub struct ProfessorContext {
    /// A mapping from student names to their grades and actors
    pub students: HashMap<String, u32>,
}

// Then we define the input event type

#[enum_dispatch]
pub trait ProfessorInput {
    // Operate on a context
    fn operate(&self, context: &mut ProfessorContext) -> Option<ProfessorOutputType>;
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct SubmitTest {
    pub answers: Answers,
    pub student: String,
}
impl ProfessorInput for SubmitTest {
    fn operate(&self, context: &mut ProfessorContext) -> Option<ProfessorOutputType> {
        let count = self.answers.answers.iter().filter(|x| **x).count();
        let student_grade = context.students.entry(self.student.clone()).or_insert(0);
        *student_grade += count as u32;
        Some(ProfessorOutputType::Grade(
            self.student.clone(),
            *student_grade,
        ))
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct AddStudent {
    pub student: String,
}
impl ProfessorInput for AddStudent {
    fn operate(&self, context: &mut ProfessorContext) -> Option<ProfessorOutputType> {
        context.students.insert(self.student.clone(), 0);
        Some(ProfessorOutputType::TestGraded)
    }
}

#[enum_dispatch(ProfessorInput)]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum ProfessorInputType {
    SubmitTest,
    AddStudent,
}

// Then we define the output event type

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum ProfessorOutputType {
    Grade(String, u32),
    TestGraded,
}

// Now generate the impls

wrapped_event!(ProfessorInputEvent, ProfessorInputType);
wrapped_event!(ProfessorOutputEvent, ProfessorOutputType);

async fn professor_event_handler(
    mut context: ProfessorContext,
    event: ProfessorInputEvent,
) -> (ProfessorContext, Option<ProfessorOutputEvent>) {
    let output = event
        .into_inner()
        .operate(&mut context)
        .map(ProfessorOutputEvent::from);
    // Finish up
    (context, output)
}

async_actor!(
    Professor,
    ProfessorInputEvent,
    ProfessorOutputEvent,
    ProfessorContext,
    professor_event_handler
);
