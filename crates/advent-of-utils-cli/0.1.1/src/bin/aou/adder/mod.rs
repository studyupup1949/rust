mod input_control;

use advent_of_utils_cli::error::{AocError, InputError};

use crate::config::AddTestConfig;

pub fn run(config: &AddTestConfig) -> Result<(), AocError> {
    // Open the input file in the editor
    if let Err(error) = input_control::edit_input(config.year, config.day, &config.database) {
        return Err(AocError::Input(InputError::TestInputFailed {
            source: Some(error),
        }));
    };

    // // Get the results
    // if let Err(error) = input_control::edit_input(config.year, config.day, &config.database) {
    //     return Err(AocError::Input(InputError::TestInputFailed {
    //         source: Some(error),
    //     }));
    // };

    Ok(())
}
