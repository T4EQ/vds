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
    name: String,

    #[serde(rename = "rm")]
    removable: bool,

    #[serde(rename = "ro")]
    read_only: bool,

    #[serde(rename = "type")]
    ty: BlockDeviceType,

    size: usize,

    mountpoints: Vec<String>,

    subsystems: String,

    #[serde(default = "default_children_node")]
    children: Vec<BlockDevice>,
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
            nix::mount::umount(&path)
                .with_context(|| format!("Unable to unmount {}", path.display()))?;
            Ok(())
        })
        .await?;
    }
    Ok(())
}

pub async fn prepare_storage_medium(path: &Path) -> anyhow::Result<()> {
    let all_block_devs = list_blockdevs().await?;

    let block_dev = all_block_devs
        .iter()
        .find(|b| b.name == *path)
        .with_context(|| format!("Block device not found: {}", path.display()))?;

    unmount_block_dev(block_dev).await?;

    // Create filesystem

    Ok(())
}
