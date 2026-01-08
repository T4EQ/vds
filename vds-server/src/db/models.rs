use std::{ffi::OsString, os::unix::ffi::OsStringExt, path::PathBuf};

use diesel::prelude::*;

use super::schema;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloadStatus {
    Pending,
    Failed(String),
    InProgress((u64, u64)),
    Downloaded(PathBuf),
}

pub const DOWNLOAD_STATUS_NOT_STARTED: i64 = 0;
pub const DOWNLOAD_STATUS_FAILED: i64 = 1;
pub const DOWNLOAD_STATUS_IN_PROGRESS: i64 = 2;
pub const DOWNLOAD_STATUS_DOWNLOADED: i64 = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Video {
    pub id: uuid::Uuid,
    pub name: String,
    pub file_size: u64,
    pub download_status: DownloadStatus,
    pub view_count: u64,
}

impl TryFrom<VideoInner> for Video {
    type Error = super::Error;
    fn try_from(value: VideoInner) -> Result<Self, super::Error> {
        let download_status = match value.download_status {
            DOWNLOAD_STATUS_NOT_STARTED => DownloadStatus::Pending,
            DOWNLOAD_STATUS_FAILED => DownloadStatus::Failed(value.message),
            DOWNLOAD_STATUS_IN_PROGRESS => {
                DownloadStatus::InProgress((value.downloaded_size as u64, value.file_size as u64))
            }
            DOWNLOAD_STATUS_DOWNLOADED => {
                DownloadStatus::Downloaded(OsString::from_vec(value.file_path).into())
            }
            v => {
                return Err(super::Error::InvalidDownloadStatus(v));
            }
        };
        Ok(Self {
            id: value.id.try_into()?,
            name: value.name,
            file_size: value.file_size as u64,
            download_status,
            view_count: value.view_count as u64,
        })
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::videos)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct VideoInner {
    pub id: String,
    pub name: String,
    pub file_size: i64,
    pub downloaded_size: i64,
    pub download_status: i64,
    pub view_count: i64,
    pub message: String,
    pub file_path: Vec<u8>,
}

#[derive(Insertable)]
#[diesel(table_name = schema::videos)]
pub struct NewVideo {
    pub id: String,
    pub name: String,
    pub file_size: i64,
}
