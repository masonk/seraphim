use failure::Backtrace;
use std::io;
use tensorflow;

pub type Result<T> = std::result::Result<T, failure::Error>;

#[derive(Debug, Fail)]
pub enum ModelError {
    #[fail(display = "Couldn't find model {} in {:?}.", name, search_dirs)]
    ModelNotFound {
        name: String,
        search_dirs: Vec<String>,
        #[fail(cause)]
        io_error: io::Error,
    },
    #[fail(display = "Couldn't load model from {:?}", dir)]
    CouldntLoad {
        dir: std::path::PathBuf,
        #[fail(cause)]
        tf: TensorflowError,
    },
}

#[derive(Debug, Fail)]
#[fail(display = "Tensorflow returned an error:\n{}", msg)]
pub struct TensorflowError {
    backtrace: Backtrace,
    code: tensorflow::Code,
    msg: String,
}

impl From<tensorflow::Status> for TensorflowError {
    fn from(status: tensorflow::Status) -> Self {
        TensorflowError {
            code: status.code(),
            msg: format!("{:?}", status),
            backtrace: failure::Backtrace::new(),
        }
    }
}

#[derive(Debug, Fail)]
pub enum SearchError {
    #[fail(display = "Invalid configuration: {}", msg)]
    InvalidConfiguration { msg: String },
    #[fail(display = "Unspecified error when searching:\n{}", msg)]
    Unspecified { msg: String, backtrace: Backtrace },
}
