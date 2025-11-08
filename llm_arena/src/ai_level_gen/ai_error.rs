use kalosm::language::*;

#[derive(Debug)]
pub enum AIError {
    /// Error in deserializing the ai response
    ResponseDeserialization { response: String },

    /// Kalosm error when running the prompt
    RunningPrompt,

    /// Serde error deserializing the response
    Serde,
}

impl From<OpenAICompatibleChatModelError> for AIError {
    fn from(err: OpenAICompatibleChatModelError) -> Self {
        AIError::RunningPrompt
    }
}

impl From<serde_json::Error> for AIError {
    fn from(err: serde_json::Error) -> Self {
        AIError::Serde
    }
}
