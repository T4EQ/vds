use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

fn default_children_node() -> Vec<BlockDevice> {
    vec![]
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockDeviceType {
    #[serde(rename = "disk")]
    Disk,

    #[serde(rename = "part")]
    Partition,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BlockDevice {
    #[serde(rename = "kname")]
    pub name: String,

    #[serde(rename = "rm")]
    pub removable: bool,

    #[serde(rename = "ro")]
    pub read_only: bool,

    #[serde(rename = "type")]
    pub ty: BlockDeviceType,

    pub size: usize,

    pub mountpoints: Vec<String>,

    pub subsystems: String,

    #[serde(default = "default_children_node")]
    pub children: Vec<BlockDevice>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LsblkOutput {
    blockdevices: Vec<BlockDevice>,
}

pub async fn list_blockdevs() -> anyhow::Result<Vec<BlockDevice>> {
    let result = Command::new("lsblk")
        .arg("--json")
        .arg("--bytes")
        .arg("--tree")
        .arg("--paths")
        .arg("-o")
        .arg("kname,type,ro,rm,mountpoints,size,subsystems")
        .output()
        .await?;

    let output: LsblkOutput = serde_json::from_slice(&result.stdout)?;

    // Filter devices
    let output: Vec<_> = output
        .blockdevices
        .into_iter()
        .filter(|d| {
            // RAM and CD-ROMs are not relevant storage mediums
            // SD cards and loop devices are also ignored.
            !d.name.starts_with("/dev/sr")
                && !d.name.starts_with("/dev/ram")
                && !d.name.starts_with("/dev/zram")
                && !d.name.starts_with("/dev/boot")
                && !d.name.starts_with("/dev/mmcblk")
                && !d.name.starts_with("/dev/loop")
        })
        .collect();

    Ok(output)
}

async fn unmount_block_dev(block_dev: &BlockDevice) -> anyhow::Result<()> {
    for child in &block_dev.children {
        // This future is boxed and pinned because of recursion.
        let fut = Box::pin(unmount_block_dev(child));
        fut.await?;
    }

    for mount_point in &block_dev.mountpoints {
        let path: PathBuf = mount_point.into();
        tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
            tracing::info!("Unmounting: {}", path.display());
            nix::mount::umount(&path)
                .with_context(|| format!("Unable to unmount {}", path.display()))?;
            Ok(())
        })
        .await??;
    }
    Ok(())
}

pub async fn prepare_storage_medium(path: &Path) -> anyhow::Result<()> {
    let all_block_devs = list_blockdevs().await?;

    tracing::info!("Preparing storage medium: {}", path.display());

    let block_dev = all_block_devs
        .iter()
        .find(|b| b.name == *path)
        .with_context(|| format!("Block device not found: {}", path.display()))?;

    unmount_block_dev(block_dev).await?;

    tracing::info!("Create ext2 fs for: {}", path.display());

    // Create filesystem
    let mke2fs_result = tokio::process::Command::new("mke2fs")
        .arg("-L")
        .arg("LEAP_DATA")
        .arg(path)
        .output()
        .await?;
    if !mke2fs_result.status.success() || mke2fs_result.status.code() != Some(0) {
        tracing::error!("Failure creating ext2 fs {mke2fs_result:?}");
        anyhow::bail!("Failure to format storage {mke2fs_result:?}");
    }

    tracing::info!("ext2fs created");
    tracing::info!("mounting file system");

    let path = path.to_owned();
    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        nix::mount::mount(
            Some(&path),
            "/var/lib/leap",
            Some("ext2"),
            nix::mount::MsFlags::MS_NOATIME,
            Option::<&str>::None,
        )?;
        Ok(())
    })
    .await??;

    tracing::info!("file system mounted");

    Ok(())
}
