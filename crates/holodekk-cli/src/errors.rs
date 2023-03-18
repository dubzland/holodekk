use std::error;
use std::fmt;
use std::io;
use std::process::Output;
use std::str::Utf8Error;

#[derive(Debug)]
pub enum HolodekkError {
    Io(io::Error),
    Utf8(Utf8Error),
    SubroutineNotFound,
    Parse(serde_json::Error),
    CommandExecution(Output),
}

impl From<io::Error> for HolodekkError {
    fn from(err: io::Error) -> HolodekkError {
        HolodekkError::Io(err)
    }
}

impl From<Utf8Error> for HolodekkError {
    fn from(err: Utf8Error) -> HolodekkError {
        HolodekkError::Utf8(err)
    }
}

impl fmt::Display for HolodekkError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HolodekkError::CommandExecution(output) => {
                write!(f, "Failed to generate manifest.\n\n")?;
                if let Ok(message) = std::str::from_utf8(&output.stderr) {
                    write!(f, "{}", message)?;
                }
                Ok(())
            },
            HolodekkError::Io(err) => write!(f, "{}", err),
            HolodekkError::Utf8(err) => write!(f, "{}", err),
            HolodekkError::Parse(err) => write!(f, "{}", err),
            HolodekkError::SubroutineNotFound => write!(f, "Unable to locate a valid subroutine."),
        }
    }
}

impl error::Error for HolodekkError {
    fn description(&self) -> &str {
        "something went wrong"
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}

