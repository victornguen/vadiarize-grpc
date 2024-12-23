mod silero;
pub mod utils;
mod vad_iter;

pub mod error;
pub mod recognizer;
pub mod tools;

pub use recognizer::Recognizer;
pub use utils::TimeStamp;
pub use utils::VadParams;

pub type OnnxSession = ort::session::Session;

pub type Result<T> = std::result::Result<T, error::VadError>;

#[cfg(test)]
mod tests {}
