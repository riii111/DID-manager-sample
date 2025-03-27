pub mod tasks;
use crate::managers::resource::ResourceManagerTrait;
use crate::managers::runtime::{FeatType, RuntimeManager};
use crate::managers::{
    resource::ResourceError,
    runtime::{RuntimeError, State},
};
use crate::state::update::tasks::{UpdateAction, UpdateActionError};
use semver::Version;
use serde_yaml::Error as SerdeYamlError;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::{self, Instant};

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

fn parse_bundles(bundles: &[PathBuf]) -> Result<Vec<UpdateAction>, UpdateError> {
    bundles
        .iter()
        .map(|bundle| {
            let yaml_content = fs::read_to_string(bundle)?;
            let update_action: UpdateAction =
                serde_yaml::from_str(&yaml_content).map_err(UpdateError::YamlParseFailed)?;
            Ok(update_action)
        })
        .collect()
}

fn get_target_state(update_error: &UpdateError) -> Option<State> {
    if update_error.requires_rollback() {
        Some(State::Rollback)
    } else if update_error.required_restore_state() {
        Some(State::Idle)
    } else {
        None
    }
}

fn extract_pending_update_actions<'b>(
    update_actions: &'b [UpdateAction],
    current_controller_version: &Version,
    current_agent_version: &Version,
) -> Result<Vec<&'b UpdateAction>, UpdateError> {
    let pending_actions: Vec<&'b UpdateAction> = update_actions
        .iter()
        .filter_map(|action| {
            let target_version = Version::parse(&action.version).ok()?;
            if *current_controller_version >= target_version
                && target_version > *current_agent_version
            {
                Some(action)
            } else {
                None
            }
        })
        .collect();

    Ok(pending_actions)
}

async fn monitor_agent_version<'a, R: RuntimeManager>(
    runtime_manager: &'a R,
    expected_version: &Version,
) -> Result<(), UpdateError> {
    let timeout = Duration::from_secs(180);
    let interval = Duration::from_secs(3);

    let start = Instant::now();
    let mut interval_timer = time::interval(interval);

    while start.elapsed() < timeout {
        interval_timer.tick().await;

        let version = runtime_manager.get_version().await.map_err(|e| {
            log::error!("Error occurred during version check: {}", e);
            UpdateError::AgentVersionCheckFailed(e.to_string())
        })?;

        if version == *expected_version {
            log::info!("Expected version received: {}", expected_version);
            return Ok(());
        } else {
            log::info!("Version did not match expected value.");
        }
    }

    Err(UpdateError::AgentVersionCheckFailed(format!(
        "Expected version '{}' was not received within {:?}.",
        expected_version, timeout
    )))
}

pub async fn execute<'a, R, T>(
    resource_manager: &'a R,
    runtime_manager: &'a mut T,
) -> Result<(), UpdateError>
where
    R: ResourceManagerTrait,
    T: RuntimeManager,
{
    log::info!("Starting update");

    let res: Result<(), UpdateError> = async {
        let current_version = Version::parse(env!("CARGO_PKG_VERSION"))
            .map_err(|_| UpdateError::InvalidVersionFormat)?;
        let runtime_info = runtime_manager.get_runtime_info()?;
        if !runtime_info.is_agent_running() {
            return Err(UpdateError::AgentNotRunning);
        }
        let current_running_agent = runtime_info.filter_by_feat(FeatType::Agent).next().unwrap();
        let bundles = resource_manager.collect_downloaded_bundles();
        let update_actions = parse_bundles(&bundles)?;
        let pending_update_actions = extract_pending_update_actions(
            &update_actions,
            &current_version,
            &current_running_agent.version,
        )?;
        for action in pending_update_actions {
            action.handle()?;
        }
        // launch new version agent
        let latest = runtime_manager.launch_agent(false)?;
        // terminate old version agents
        runtime_manager.kill_other_agents(latest.process_id)?;
        monitor_agent_version(runtime_manager, &current_version).await?;
        // if you test for rollback, comment out a follow line.
        resource_manager.remove()?;
        Ok(())
    }
    .await;

    match res {
        Ok(()) => runtime_manager.update_state(crate::managers::runtime::State::Idle)?,
        Err(update_error) => {
            if let Some(target_state) = get_target_state(&update_error) {
                runtime_manager.update_state(target_state)?;
            }
            return Err(update_error);
        }
    }

    log::info!("Update completed");

    Ok(())
}
