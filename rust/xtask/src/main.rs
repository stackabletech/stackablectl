use clap::Parser;
use snafu::Snafu;

use crate::{
    completions::GenCompError, docs::GenDocsError, man::GenManError, readme::GenReadmeError,
};

mod completions;
mod docs;
mod man;
mod readme;

#[derive(clap::Parser)]
#[allow(clippy::enum_variant_names)]
enum XtaskCommand {
    GenAll,
    GenMan,
    GenComp,
    GenCtlReadme,
    GenDocs,
}

#[derive(Debug, Snafu)]
enum TaskError {
    #[snafu(display("failed to generate man pages"), context(false))]
    Man { source: GenManError },

    #[snafu(display("failed to generate shell completions"), context(false))]
    Comp { source: GenCompError },

    #[snafu(
        display("failed to generate stackablectl usage README file"),
        context(false)
    )]
    Readme { source: GenReadmeError },

    #[snafu(display("failed to generate stackablectl doc pages"), context(false))]
    Docs { source: GenDocsError },
}

#[snafu::report]
fn main() -> Result<(), TaskError> {
    match XtaskCommand::parse() {
        XtaskCommand::GenAll => {
            // IMPORTANT (@NickLarsenNZ): ensure all commands defined below are also in here.
            man::generate()?;
            completions::generate()?;
            readme::generate()?;
            docs::generate()?;
        }
        XtaskCommand::GenMan => man::generate()?,
        XtaskCommand::GenComp => completions::generate()?,
        XtaskCommand::GenCtlReadme => readme::generate()?,
        XtaskCommand::GenDocs => docs::generate()?,
    }

    Ok(())
}
