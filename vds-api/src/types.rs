//! Common data types used by the APIs

/// Download progress. A number from 0 to 1, where 1 indicates completed and 0 not
/// started.
#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Clone)]
pub struct Progress(pub f64);

/// The status of the video download
#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Clone)]
pub enum VideoStatus {
    /// The video download has not started
    Pending,
    /// The video download is in progress
    Downloading(Progress),
    /// The video download is completed
    Downloaded,
    /// The video download failed
    Failed(String),
}

/// Metadata of a single video of the local server.
#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Clone)]
pub struct LocalVideoMeta {
    /// Unique identifier of the video
    pub id: String,
    /// Human-readable name of the video
    pub name: String,
    /// Size of the video in bytes
    pub size: usize,
    /// Download status
    pub status: VideoStatus,
}

/// Metadata of a single video present in the remote server.
#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq, Clone)]
pub struct RemoteVideoMeta {
    /// Unique identifier of the video
    pub id: String,
    /// Human-readable name of the video
    pub name: String,
    /// Size of the video in bytes
    pub size: usize,
    /// flag indicating whether the video is also locally cached.
    pub local: bool,
}
