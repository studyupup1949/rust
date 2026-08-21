use std::{fmt::Display, fs::File, io::Write};

use indoc::{formatdoc, indoc};

use crate::model::{CStandard, CppStandard, Language};

const EXAMPLE_C_PROGRAM: &'static str = indoc! {r#"
#include <stdio.h>

int main()
{
    printf("Hello World!");
}
"#};

const EXAMPLE_CPP_PROGRAM: &'static str = indoc! {r#"
#include <iostream>

using std::cout;

int main()
{
    cout << "Hello World!";
}
"#};

pub fn generate_project(
    path: String,
    project_name: String,
    lang: Language,
) -> Result<(), std::io::Error> {
    let cmakelists_path = format!("{path}/CMakeLists.txt");
    let mut cmakelists_file = File::create(cmakelists_path)?;

    match lang {
        Language::C(standard) => {
            let example_path = format!("{path}/main.c");
            let mut example_file = File::create(example_path)?;

            cmakelists_file.write_all(get_c_cmakelists(&project_name, standard).as_bytes())?;
            example_file.write_all(EXAMPLE_C_PROGRAM.as_bytes())?;
        }
        Language::CPP(standard) => {
            let example_path = format!("{path}/main.cpp");
            let mut example_file = File::create(example_path)?;

            cmakelists_file.write_all(get_cpp_cmakelists(&project_name, standard).as_bytes())?;
            example_file.write_all(EXAMPLE_CPP_PROGRAM.as_bytes())?;
        }
    }
    Ok(())
}

fn get_c_cmakelists(project_name: &String, standard: CStandard) -> String {
    get_common_cmakelists(
        project_name,
        format!("set(CMAKE_C_STANDARD {standard})"),
        "main.cpp",
    )
}

fn get_cpp_cmakelists(project_name: &String, standard: CppStandard) -> String {
    get_common_cmakelists(
        project_name,
        format!("set(CMAKE_CXX_STANDARD {standard})"),
        "main.cpp",
    )
}

fn get_common_cmakelists<Env: Display>(project_name: &str, env: Env, exec: &str) -> String {
    formatdoc! {r#"
        cmake_minimum_required(VERSION 3.21)

        project({project_name})

        {env}

        add_executable({project_name} {exec})
    "#}
}
