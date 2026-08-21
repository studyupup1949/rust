use actm::prelude::*;
use nanorand::Rng;

use crate::{Answers, Test};

// First we define the context type

pub struct StudentContext {
    /// My name
    pub name: String,
}

// Then we define the input event type

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct StudentInput {
    /// The test the student is to perform
    pub test: Test,
}

// Then the output event
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct StudentOutput {
    /// The answers to the test that the student just completed
    pub answers: Answers,
    /// The student's name
    pub name: String,
}

// Generate the impls

wrapped_event!(StudentInputEvent, StudentInput);
wrapped_event!(StudentOutputEvent, StudentOutput);

async fn student_event_handler(
    context: StudentContext,
    event: StudentInputEvent,
) -> (StudentContext, Option<StudentOutputEvent>) {
    let mut rng = nanorand::tls_rng();
    let results = event
        .into_inner()
        .test
        .questions
        .into_iter()
        .map(|x| -> bool { x > rng.generate_range(1_u32..=100) })
        .collect::<Vec<_>>();
    let name = context.name.clone();
    let new_event: StudentOutputEvent = StudentOutput {
        answers: Answers { answers: results },
        name,
    }
    .into();
    (context, Some(new_event))
}

async_actor!(
    Student,
    StudentInputEvent,
    StudentOutputEvent,
    StudentContext,
    student_event_handler
);
