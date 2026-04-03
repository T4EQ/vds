use crate::cfg::LeapConfig;
use leap_api::types::NetworkConfig;
use leap_api::types::ProvisionStatus;

use std::path::{Path, PathBuf};

mod cfg;
mod network;
mod storage;

pub use storage::{BlockDevice, BlockDeviceType};

#[derive(Debug)]
pub struct NetworkStep {}

#[derive(Debug)]
pub struct StorageStep {
    network_config: NetworkConfig,
}

#[derive(Debug)]
pub struct LeapConfigStep {
    network_config: NetworkConfig,
    storage_node: PathBuf,
}

#[derive(Debug, serde::Serialize)]
pub struct CompleteStep {
    network_config: NetworkConfig,
    storage_node: PathBuf,
    configuration: LeapConfig,
}

/// Sealed marker trait
mod private {
    pub trait ProvisionStep {}
}
use private::ProvisionStep;

impl ProvisionStep for NetworkStep {}
impl ProvisionStep for StorageStep {}
impl ProvisionStep for LeapConfigStep {}
impl ProvisionStep for CompleteStep {}

#[derive(Debug)]
pub struct Provision<Step: ProvisionStep> {
    inner: Step,
}

impl<S: ProvisionStep> Provision<S> {
    /// Constructs the Provision in its default state
    pub const fn new() -> Provision<NetworkStep> {
        Provision {
            inner: NetworkStep {},
        }
    }

    /// Listing the block devices can be done at any step
    pub async fn list_blockdevs(&self) -> anyhow::Result<Vec<storage::BlockDevice>> {
        storage::list_blockdevs().await
    }
}

impl Provision<NetworkStep> {
    pub async fn configure_network(
        self,
        network_config: &NetworkConfig,
    ) -> Result<Provision<StorageStep>, (anyhow::Error, Provision<NetworkStep>)> {
        // TODO: this is not yet implemented
        Ok(Provision {
            inner: StorageStep {
                network_config: network_config.clone(),
            },
        })
    }
}

impl Provision<StorageStep> {
    pub async fn revert(self) -> Provision<NetworkStep> {
        // TODO: need to roll back some networking actions?
        Provision {
            inner: NetworkStep {},
        }
    }

    pub async fn configure_storage(
        self,
        path: &Path,
    ) -> Result<Provision<LeapConfigStep>, (anyhow::Error, Provision<StorageStep>)> {
        if let Err(err) = storage::prepare_storage_medium(path).await {
            Err((err, self))
        } else {
            Ok(Provision {
                inner: LeapConfigStep {
                    network_config: self.inner.network_config,
                    storage_node: path.to_path_buf(),
                },
            })
        }
    }
}

impl Provision<LeapConfigStep> {
    pub async fn revert(self) -> Provision<StorageStep> {
        Provision {
            inner: StorageStep {
                network_config: self.inner.network_config,
            },
        }
    }

    pub async fn configure_leap(
        self,
        config: &leap_api::provision::config::post::LeapConfig,
    ) -> Result<Provision<CompleteStep>, (anyhow::Error, Provision<LeapConfigStep>)> {
        match cfg::provision_configuration(config).await {
            Ok(configuration) => Ok(Provision {
                inner: CompleteStep {
                    network_config: self.inner.network_config,
                    storage_node: self.inner.storage_node,
                    configuration,
                },
            }),
            Err(error) => Err((error, self)),
        }
    }
}

impl Provision<CompleteStep> {
    pub async fn finish(self) -> anyhow::Result<()> {
        tokio::fs::write(
            "/var/lib/leap/provision.json",
            &serde_json::to_vec(&self.inner)?,
        )
        .await?;
        loop {
            nix::sys::reboot::reboot(nix::sys::reboot::RebootMode::RB_AUTOBOOT)?;
        }
    }
}

#[allow(
    clippy::large_enum_variant,
    reason = "No improvement would be made here if we box each variant"
)]
enum DynProvisionImpl {
    Network(Provision<NetworkStep>),
    Storage(Provision<StorageStep>),
    LeapConfig(Provision<LeapConfigStep>),
    Complete(Provision<CompleteStep>),
}

trait ProvisionVariant: ProvisionStep + Sized {
    const CONSTRUCTOR: fn(Provision<Self>) -> DynProvisionImpl;
}

impl<S> From<Provision<S>> for DynProvisionImpl
where
    S: ProvisionVariant + ProvisionStep,
{
    fn from(value: Provision<S>) -> Self {
        <S as ProvisionVariant>::CONSTRUCTOR(value)
    }
}

impl ProvisionVariant for NetworkStep {
    const CONSTRUCTOR: fn(Provision<Self>) -> DynProvisionImpl = DynProvisionImpl::Network;
}

impl ProvisionVariant for StorageStep {
    const CONSTRUCTOR: fn(Provision<Self>) -> DynProvisionImpl = DynProvisionImpl::Storage;
}

impl ProvisionVariant for LeapConfigStep {
    const CONSTRUCTOR: fn(Provision<Self>) -> DynProvisionImpl = DynProvisionImpl::LeapConfig;
}

impl ProvisionVariant for CompleteStep {
    const CONSTRUCTOR: fn(Provision<Self>) -> DynProvisionImpl = DynProvisionImpl::Complete;
}

pub struct DynProvision(Option<DynProvisionImpl>);

impl DynProvision {
    pub const fn new() -> Self {
        DynProvision(Some(DynProvisionImpl::Network(
            Provision::<NetworkStep>::new(),
        )))
    }

    /// Listing the block devices can be done at any step
    pub async fn list_blockdevs(&self) -> anyhow::Result<Vec<storage::BlockDevice>> {
        match self.0.as_ref() {
            Some(DynProvisionImpl::Network(p)) => p.list_blockdevs().await,
            Some(DynProvisionImpl::Storage(p)) => p.list_blockdevs().await,
            Some(DynProvisionImpl::LeapConfig(p)) => p.list_blockdevs().await,
            Some(DynProvisionImpl::Complete(p)) => p.list_blockdevs().await,
            None => {
                anyhow::bail!("BUG: No inner DynProvisionImpl");
            }
        }
    }

    pub async fn configure_network(
        &mut self,
        network_config: &NetworkConfig,
    ) -> anyhow::Result<()> {
        match self.0.take() {
            Some(DynProvisionImpl::Network(p)) => {
                self.handle_retval(p.configure_network(network_config).await)?
            }
            Some(DynProvisionImpl::Storage(p)) => {
                self.handle_retval(p.revert().await.configure_network(network_config).await)?;
            }
            Some(v) => {
                self.0.replace(v);
                anyhow::bail!("configure_network called on invalid state");
            }
            None => {
                anyhow::bail!("BUG: No inner DynProvisionImpl");
            }
        }
        Ok(())
    }

    pub async fn configure_storage(&mut self, device_path: &Path) -> anyhow::Result<()> {
        match self.0.take() {
            Some(DynProvisionImpl::Storage(p)) => {
                self.handle_retval(p.configure_storage(device_path).await)?;
            }
            Some(DynProvisionImpl::LeapConfig(p)) => {
                self.handle_retval(p.revert().await.configure_storage(device_path).await)?;
            }
            Some(v) => {
                self.0.replace(v);
                anyhow::bail!("configure_storage called on invalid state");
            }
            None => {
                anyhow::bail!("BUG: No inner DynProvisionImpl");
            }
        }
        Ok(())
    }

    pub async fn configure_leap(
        &mut self,
        config: &leap_api::provision::config::post::LeapConfig,
    ) -> anyhow::Result<()> {
        match self.0.take() {
            Some(DynProvisionImpl::LeapConfig(p)) => {
                self.handle_retval(p.configure_leap(config).await)?;
            }
            Some(v) => {
                self.0.replace(v);
                anyhow::bail!("configure_leap called on invalid state");
            }
            None => {
                anyhow::bail!("BUG: No inner DynProvisionImpl");
            }
        }
        Ok(())
    }

    pub async fn finish(&mut self) -> anyhow::Result<()> {
        match self.0.take() {
            Some(DynProvisionImpl::Complete(p)) => {
                p.finish().await?;
                unreachable!("Finish should have rebooted the system");
            }
            Some(v) => {
                self.0.replace(v);
                anyhow::bail!("finish called on invalid state");
            }
            None => {
                anyhow::bail!("BUG: No inner DynProvisionImpl");
            }
        }
    }

    pub fn status(&self) -> anyhow::Result<ProvisionStatus> {
        match self.0.as_ref() {
            Some(DynProvisionImpl::Network(_)) => Ok(ProvisionStatus::NetworkConfig),
            Some(DynProvisionImpl::Storage(_)) => Ok(ProvisionStatus::StorageConfig),
            Some(DynProvisionImpl::LeapConfig(_)) => Ok(ProvisionStatus::LeapConfig),
            Some(DynProvisionImpl::Complete(_)) => Ok(ProvisionStatus::Completed),
            None => {
                anyhow::bail!("BUG: No inner DynProvisionImpl");
            }
        }
    }

    fn handle_retval<Src: ProvisionVariant, Dst: ProvisionVariant>(
        &mut self,
        result: Result<Provision<Dst>, (anyhow::Error, Provision<Src>)>,
    ) -> anyhow::Result<()> {
        match result {
            Ok(dst) => {
                self.0.replace(DynProvisionImpl::from(dst));
                Ok(())
            }
            Err((err, src)) => {
                self.0.replace(DynProvisionImpl::from(src));
                Err(err)
            }
        }
    }
}

impl std::fmt::Debug for DynProvision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Some(DynProvisionImpl::Network(p)) => write!(f, "{p:?}"),
            Some(DynProvisionImpl::Storage(p)) => write!(f, "{p:?}"),
            Some(DynProvisionImpl::LeapConfig(p)) => write!(f, "{p:?}"),
            Some(DynProvisionImpl::Complete(p)) => write!(f, "{p:?}"),
            None => write!(f, "ProvisionInvalidState"),
        }
    }
}
