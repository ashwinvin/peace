use std::str::FromStr;

use crate::output::CliProgressFormatOptParseError;

/// How to format progress on the CLI.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CliProgressFormatOpt {
    /// Automatically detect whether to render a progress bar or the output
    /// format.
    Auto,
    /// Render progress in the same format as the output.
    Output,
    /// Always render progress as a progress bar.
    ProgressBar,
}

impl FromStr for CliProgressFormatOpt {
    type Err = CliProgressFormatOptParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "auto" => Ok(Self::Auto),
            "output" => Ok(Self::Output),
            "pb" | "progress_bar" => Ok(Self::ProgressBar),
            _ => Err(CliProgressFormatOptParseError(s.to_string())),
        }
    }
}
