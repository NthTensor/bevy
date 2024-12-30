use bevy_error::{Advice, Result};
use thiserror::Error;

#[derive(Error, Debug, Advice)]
enum MyDiagnostic {
    #[error("error case A")]
    CaseA,
}

fn system() -> Result {
    Err(MyDiagnostic::CaseA)?;
    Ok(())
}
