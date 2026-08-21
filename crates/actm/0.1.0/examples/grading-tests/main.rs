//! Simple example of students taking tests given by the professor

use std::collections::HashMap;

use actm::{executor::AsyncStd, prelude::*, traits::ActorExt};
use futures::future::join_all;
use nanorand::Rng;
use professor::ProfessorOutputType;

use crate::{
    professor::{
        AddStudent, Professor, ProfessorContext, ProfessorInputType, ProfessorOutputEvent,
        SubmitTest,
    },
    student::{Student, StudentContext, StudentInput, StudentOutputEvent},
    university::{University, UniversityContext, UniversityInput},
};

pub mod professor;
pub mod student;
pub mod university;

/// A dummy test structure
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Test {
    /// A question is the probability out of 100 that a student will answer correctly, or `true`
    questions: Vec<u32>,
}

/// A dummy answers structure
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Answers {
    /// An answer of `true` is correct
    answers: Vec<bool>,
}

#[async_std::main]
async fn main() {
    // Spawn our professor
    println!("Spawning professor");
    let professor = Professor::<AsyncStd>::new(
        ProfessorContext {
            students: HashMap::new(),
        },
        None,
    );
    // Create a university and force it to listen to our professor
    let university = University::<AsyncStd>::new(UniversityContext::default(), None);
    professor
        .register_consumer(Adaptor::wrap(
            |e: ProfessorOutputEvent| {
                let e = e.into_inner();
                match e {
                    ProfessorOutputType::Grade(name, grade) => {
                        Some(UniversityInput::Grade(name, grade).into())
                    }
                    _ => None,
                }
            },
            university.clone(),
        ))
        .await
        .unwrap();
    professor.catchup().wait().await;
    // Spawn some students
    println!("Spawning students");
    let names = ["Alice", "Bob", "Eve", "Mallory", "Tim"];
    let students = join_all(names.into_iter().map(|name| {
        let professor = professor.clone();
        async move {
            // Spawn up the student
            let student = Student::<AsyncStd>::new(
                StudentContext {
                    name: name.to_string(),
                },
                None,
            );
            // Connect it to the professor
            println!("Hooking {name} up to professor");
            student
                .register_consumer(Adaptor::wrap(
                    |e: StudentOutputEvent| {
                        let e = e.into_inner();
                        Some(
                            ProfessorInputType::SubmitTest(SubmitTest {
                                answers: e.answers,
                                student: e.name,
                            })
                            .into(),
                        )
                    },
                    professor.clone(),
                ))
                .await
                .unwrap();
            // Tell the professor about it
            println!("Telling professor about {name}");
            professor
                .call(
                    ProfessorInputType::AddStudent(AddStudent {
                        student: name.to_string(),
                    })
                    .into(),
                )
                .await
                .unwrap();
            // Return it
            println!("Setup {name}");
            student
        }
    }))
    .await;
    // Generate a few tests
    for i in 0..5 {
        println!("Generating test {i}");
        let test = Test {
            questions: (0..100)
                .map(|_| nanorand::tls_rng().generate_range(1_u32..=100))
                .collect(),
        };
        // Send the students a test
        println!("Prompting students {i}");
        join_all(students.iter().map(|student| async {
            student
                .call(StudentInput { test: test.clone() }.into())
                .await
                .unwrap();
        }))
        .await;
    }
    println!("Making students submit tests");
    // Force all the students to submit their tests
    join_all(students.iter().map(|student| async {
        student.catchup().wait().await;
    }))
    .await;
    println!("Making professor grade tests");
    // Force the professor to accept all the exams
    professor.catchup().wait().await;
    println!("All tests graded");
    // Force the university to have our results
    university.catchup().wait().await;
    // Collect the results from our students
    let grades = university
        .call(UniversityInput::Get.into())
        .await
        .unwrap()
        .unwrap()
        .into_inner()
        .0;
    println!("{:#?}", grades);
}
