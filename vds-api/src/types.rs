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
    /// Total views of the video
    pub view_count: u64,
}

/// Grouped section of video content
#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Clone)]
pub struct GroupedSection {
    /// Name of the section
    pub name: String,

    /// Content within the section. Ordered as displayed
    pub content: Vec<LocalVideoMeta>,
}
