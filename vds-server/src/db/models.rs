use std::{ffi::OsString, os::unix::ffi::OsStringExt, path::PathBuf};

use diesel::{
    prelude::*,
    sql_types::{BigInt, Binary, Text},
};

use super::schema;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloadStatus {
    Pending,
    Failed(String),
    InProgress((u64, u64)),
    Downloaded(PathBuf),
}

impl DownloadStatus {
    pub fn is_downloaded(&self) -> bool {
        matches!(self, DownloadStatus::Downloaded(_))
    }
}

impl Selectable<diesel::sqlite::Sqlite> for DownloadStatus {
    type SelectExpression = (
        schema::videos::dsl::file_size,
        schema::videos::dsl::downloaded_size,
        schema::videos::dsl::download_status,
        schema::videos::dsl::message,
        schema::videos::dsl::file_path,
    );

    fn construct_selection() -> Self::SelectExpression {
        (
            schema::videos::dsl::file_size,
            schema::videos::dsl::downloaded_size,
            schema::videos::dsl::download_status,
            schema::videos::dsl::message,
            schema::videos::dsl::file_path,
        )
    }
}

impl Queryable<(BigInt, BigInt, BigInt, Text, Binary), diesel::sqlite::Sqlite> for DownloadStatus {
    type Row = (i64, i64, i64, String, Vec<u8>);

    fn build(
        (file_size, downloaded_size, download_status, message, file_path): Self::Row,
    ) -> diesel::deserialize::Result<Self> {
        Ok(match download_status {
            DOWNLOAD_STATUS_NOT_STARTED => DownloadStatus::Pending,
            DOWNLOAD_STATUS_FAILED => DownloadStatus::Failed(message),
            DOWNLOAD_STATUS_IN_PROGRESS => {
                DownloadStatus::InProgress((downloaded_size as u64, file_size as u64))
            }
            DOWNLOAD_STATUS_DOWNLOADED => {
                DownloadStatus::Downloaded(OsString::from_vec(file_path).into())
            }
            v => {
                return Err(super::Error::InvalidDownloadStatus(v).into());
            }
        })
    }
}

pub const DOWNLOAD_STATUS_NOT_STARTED: i64 = 0;
pub const DOWNLOAD_STATUS_FAILED: i64 = 1;
pub const DOWNLOAD_STATUS_IN_PROGRESS: i64 = 2;
pub const DOWNLOAD_STATUS_DOWNLOADED: i64 = 3;

#[derive(Queryable, Debug, Clone, PartialEq, Eq)]
#[diesel(table_name = schema::videos)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Video {
    #[diesel(deserialize_as = String)]
    pub id: uuid::Uuid,

    pub name: String,

    #[diesel(deserialize_as = i64)]
    pub file_size: u64,

    pub download_status: DownloadStatus,

    #[diesel(deserialize_as = i64)]
    pub view_count: u64,
}

impl Selectable<diesel::sqlite::Sqlite> for Video {
    type SelectExpression = (
        schema::videos::dsl::id,
        schema::videos::dsl::name,
        schema::videos::dsl::file_size,
        <DownloadStatus as Selectable<diesel::sqlite::Sqlite>>::SelectExpression,
        schema::videos::dsl::view_count,
    );

    fn construct_selection() -> Self::SelectExpression {
        (
            schema::videos::dsl::id,
            schema::videos::dsl::name,
            schema::videos::dsl::file_size,
            <DownloadStatus as Selectable<diesel::sqlite::Sqlite>>::construct_selection(),
            schema::videos::dsl::view_count,
        )
    }
}

#[derive(Insertable)]
#[diesel(table_name = schema::videos)]
pub struct NewVideo {
    pub id: String,
    pub name: String,
    pub file_size: i64,
}
