pub mod tasks;
use crate::managers::{resource::ResourceError, runtime::RuntimeError};
use crate::state::update::tasks::UpdateActionError;
use serde_yaml::Error as SerdeYamlError;

#[derive(Debug, thiserror::Error)]
pub enum UpdateError {
    #[error("Failed to find bundle")]
    BundleNotFound,
    #[error("Invalid version format")]
    InvalidVersionFormat,
    #[error("update action error: {0}")]
    UpdateActionFailed(#[from] UpdateActionError),
    #[error("Failed to read YAML file: {0}")]
    YamlReadFailed(#[from] std::io::Error),
    #[error("Failed to parse YAML: {0}")]
    YamlParseFailed(#[source] SerdeYamlError),
    #[error("Failed to update state: {0}")]
    UpdateStateFailed(#[source] RuntimeError),
    #[error("Failed to Agent version check: {0}")]
    AgentVersionCheckFailed(String),
    #[error("runtime operation failed: {0}")]
    RuntimeError(#[from] RuntimeError),
    #[error("resource operation failed: {0}")]
    ResourceError(#[from] ResourceError),
    #[error("Agent not running")]
    AgentNotRunning,
}

impl UpdateError {
    pub fn required_restore_state(&self) -> bool {
        !matches!(self, UpdateError::AgentNotRunning)
    }

    pub fn requires_rollback(&self) -> bool {
        !matches!(
            self,
            UpdateError::ResourceError(ResourceError::RemoveFailed(_))
        )
    }
}
