use rusqlite::params;
use std::fmt::Display;

use crate::{
    error::{AocError, DatabaseError},
    Parts,
};

use super::AocDatabase;

#[derive(Debug, Clone, Copy)]
enum Query {
    Insert,
    Select,
}

#[derive(Debug, Clone, Copy)]
enum Table {
    Inputs,
    Results,
    TestResults,
}

impl Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Table::Inputs => write!(f, "Inputs"),
            Table::Results => write!(f, "Results"),
            Table::TestResults => write!(f, "Test_Results"),
        }
    }
}

impl AocDatabase {
    fn buld_query(&self, table: Table, field: &str, query_type: Query) -> String {
        match query_type {
            Query::Insert => {
                format!(
                    "INSERT INTO {table} (year, day, {field})
                     VALUES (?1, ?2, ?3)
                     ON CONFLICT(year, day) DO UPDATE SET
                     {field} = ?3"
                )
            }
            Query::Select => {
                format!(
                    "SELECT {field}
                     FROM {table}
                     WHERE year = ?1 AND day = ?2",
                )
            }
        }
    }

    pub(super) fn create_inputs(&self) -> Result<(), AocError> {
        self.execute(
            "CREATE TABLE IF NOT EXISTS Inputs (
                year INTEGER NOT NULL,
                day INTEGER NOT NULL,
                input TEXT,
                test_input TEXT,
                PRIMARY KEY(year, day)
            )",
            [],
        )?;

        Ok(())
    }

    pub fn get_input(&self, year: i32, day: u8, test: bool) -> Result<String, AocError> {
        let field = match test {
            true => "test_input",
            false => "input",
        };
        let query = self.buld_query(Table::Inputs, field, Query::Select);
        let conn = self.get_conn()?;
        let res = conn.query_row(&query, [year, day.into()], |row| row.get(0));
        match res {
            Ok(s) => Ok(s),
            Err(error) => Err(AocError::Database(DatabaseError::DatabaseQuerying {
                object: field.to_string(),
                source: error,
            })),
        }
    }

    pub fn set_input(&self, year: i32, day: u8, test: bool, input: String) -> Result<(), AocError> {
        let field = match test {
            true => "test_input",
            false => "input",
        };
        let query = self.buld_query(Table::Inputs, field, Query::Insert);
        self.execute(&query, params![year, day, input])?;
        Ok(())
    }

    pub fn has_input(&self, year: i32, day: u8, test: bool) -> Result<bool, AocError> {
        let field = match test {
            true => "test_input",
            false => "input",
        };
        let conn = self.get_conn()?;
        match conn.query_row(
            &format!(
                "SELECT COUNT({field}) FROM Inputs 
                 WHERE year = ? AND day = ? 
                 AND {field} IS NOT NULL"
            ),
            [year, day.into()],
            |row| row.get::<usize, i32>(0),
        ) {
            Ok(count) => Ok(count > 0),
            Err(error) => Err(AocError::Database(DatabaseError::DatabaseQuerying {
                object: "Input".to_string(),
                source: error,
            })),
        }
    }

    pub(super) fn create_results(&self) -> Result<(), AocError> {
        self.execute(
            "CREATE TABLE IF NOT EXISTS Results (
                year INTEGER NOT NULL,
                day INTEGER NOT NULL,
                part_1 TEXT,
                part_2 TEXT,
                PRIMARY KEY(year, day)
            )",
            [],
        )?;

        Ok(())
    }

    pub(super) fn create_test_results(&self) -> Result<(), AocError> {
        self.execute(
            "CREATE TABLE IF NOT EXISTS Test_Results (
                year INTEGER NOT NULL,
                day INTEGER NOT NULL,
                part_1 TEXT,
                part_2 TEXT,
                PRIMARY KEY(year, day)
            )",
            [],
        )?;

        Ok(())
    }

    pub fn get_results(
        &self,
        year: i32,
        day: u8,
        part: Parts,
        test: bool,
    ) -> Result<String, AocError> {
        let field = match part {
            Parts::Part1 => "part_1",
            Parts::Part2 => "part_2",
        };
        let table = match test {
            true => Table::TestResults,
            false => Table::Results,
        };
        let query = self.buld_query(table, field, Query::Select);
        let conn = self.get_conn()?;
        let res = conn.query_row(&query, [year, day.into()], |row| row.get(0));
        match res {
            Ok(s) => Ok(s),
            Err(error) => Err(AocError::Database(DatabaseError::DatabaseQuerying {
                object: field.to_string(),
                source: error,
            })),
        }
    }

    pub fn set_result(
        &self,
        year: i32,
        day: u8,
        test: bool,
        part: Parts,
        result: String,
    ) -> Result<(), AocError> {
        let field = match part {
            Parts::Part1 => "part_1",
            Parts::Part2 => "part_2",
        };
        let table = match test {
            true => Table::TestResults,
            false => Table::Results,
        };
        let query = self.buld_query(table, field, Query::Insert);
        self.execute(&query, params![year, day, result])?;
        Ok(())
    }

    pub fn has_result(
        &self,
        year: i32,
        day: u8,
        part: Parts,
        test: bool,
    ) -> Result<bool, AocError> {
        let table = match test {
            true => "Test_Results",
            false => "Results",
        };

        let field = match part {
            Parts::Part1 => "part_1",
            Parts::Part2 => "part_2",
        };

        let conn = self.get_conn()?;
        match conn.query_row(
            &format!(
                "SELECT COUNT({field}) FROM {table} 
                 WHERE year = ? AND day = ? 
                 AND {field} IS NOT NULL"
            ),
            [year, day.into()],
            |row| row.get::<usize, i32>(0),
        ) {
            Ok(count) => Ok(count > 0),
            Err(error) => Err(AocError::Database(DatabaseError::DatabaseQuerying {
                object: "Input".to_string(),
                source: error,
            })),
        }
    }
}
