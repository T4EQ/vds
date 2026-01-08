mod models;
mod schema;

use std::path::Path;
use std::time::Duration;

pub use models::{DownloadStatus, Video};

use deadpool_diesel::{Manager, Pool};
use diesel::{connection::SimpleConnection, prelude::*};

use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

use crate::db::models::VideoInner;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Pool error: {0:?}")]
    Pool(#[from] deadpool_diesel::PoolError),
    #[error("Build error: {0:?}")]
    Build(#[from] deadpool_diesel::sqlite::BuildError),
    #[error("Diesel error: {0:?}")]
    Diesel(#[from] diesel::result::Error),
    #[error("Migration error")]
    Migration,
    #[error("Invalid download status: {0:?}")]
    InvalidDownloadStatus(i64),
    #[error("Invalid uuid: {0:?}")]
    InvalidUUID(#[from] uuid::Error),
}

pub type Result<T> = core::result::Result<T, Error>;

pub struct Database {
    pool: Pool<Manager<diesel::sqlite::SqliteConnection>>,
}

impl Database {
    pub async fn open(url: &str, timeout_ms: Duration) -> Result<Self> {
        let manager = Manager::new(url, deadpool_diesel::Runtime::Tokio1);
        let pool: Pool<Manager<_>> = Pool::builder(manager)
            .max_size(16)
            .post_create(deadpool_diesel::sqlite::Hook::sync_fn(move |c, _m| {
                let mut c = c.lock().expect("poisoned mutex");
                c.batch_execute("PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;")
                    .expect("Unable to configure journal mode on sqlite DB connection");
                c.batch_execute(&format!(
                    "PRAGMA busy_timeout = {};",
                    timeout_ms.as_millis()
                ))
                .expect("Unable to set busy timeout on DB connection");
                Ok(())
            }))
            .build()?;

        Ok(Self { pool })
    }

    pub async fn apply_pending_migrations(&self) -> Result<()> {
        let connection = self.pool.get().await?;
        connection
            .interact(move |conn| {
                conn.run_pending_migrations(MIGRATIONS)
                    .map_err(|_| Error::Migration)?;
                Ok(())
            })
            .await
            .expect("Unexpected panic of a background DB thread")
    }

    pub async fn find_video(&self, req_id: uuid::Uuid) -> Result<Video> {
        let req_id = req_id.to_string();

        let connection = self.pool.get().await?;
        connection
            .interact(move |conn| {
                use schema::videos::dsl;

                let video = dsl::videos
                    .filter(dsl::id.eq(&req_id))
                    .select(models::VideoInner::as_select())
                    .get_result(conn)?;
                video.try_into()
            })
            .await
            .expect("Unexpected panic of a background DB thread")
    }

    pub async fn delete_video(&self, req_id: uuid::Uuid) -> Result<()> {
        use schema::videos::dsl::*;

        let req_id = req_id.to_string();

        let connection = self.pool.get().await?;
        connection
            .interact(move |c| {
                diesel::delete(videos.filter(id.eq(req_id))).execute(c)?;
                Ok(())
            })
            .await
            .expect("Unexpected panic of a background DB thread")
    }

    pub async fn insert_video(&self, id: uuid::Uuid, name: &str, file_size: u64) -> Result<()> {
        let id = id.to_string();
        let new_vid = models::NewVideo {
            id,
            name: name.to_string(),
            file_size: file_size as i64,
        };

        let connection = self.pool.get().await?;
        connection
            .interact(move |c| {
                diesel::insert_into(schema::videos::dsl::videos)
                    .values(new_vid)
                    .execute(c)?;
                Ok(())
            })
            .await
            .expect("Unexpected panic of a background DB thread")
    }

    pub async fn increment_view_count(&self, req_id: uuid::Uuid) -> Result<Video> {
        let connection = self.pool.get().await?;
        connection
            .interact(move |c| -> Result<Video> {
                use schema::videos::dsl;
                let v: VideoInner = diesel::update(dsl::videos.find(req_id.to_string()))
                    .set((dsl::view_count.eq(dsl::view_count + 1),))
                    .get_result(c)?;
                v.try_into()
            })
            .await
            .expect("Unexpected panic of a background DB thread")
    }

    pub async fn update_download_progress(
        &self,
        req_id: uuid::Uuid,
        downloaded_size: u64,
    ) -> Result<()> {
        let connection = self.pool.get().await?;
        connection
            .interact(move |c| {
                use schema::videos::dsl;
                diesel::update(dsl::videos.find(req_id.to_string()))
                    .set((
                        dsl::download_status.eq(models::DOWNLOAD_STATUS_IN_PROGRESS),
                        dsl::downloaded_size.eq(downloaded_size as i64),
                        dsl::message.eq(""),
                    ))
                    .execute(c)?;
                Ok(())
            })
            .await
            .expect("Unexpected panic of a background DB thread")
    }

    pub async fn set_download_failed(&self, req_id: uuid::Uuid, message: &str) -> Result<()> {
        let message = message.to_string(); // Need a copy since interact runs on a separate thread
        // and requires 'static.

        let connection = self.pool.get().await?;
        connection
            .interact(move |c| {
                use schema::videos::dsl;
                diesel::update(dsl::videos.find(req_id.to_string()))
                    .set((
                        dsl::download_status.eq(models::DOWNLOAD_STATUS_FAILED),
                        dsl::message.eq(message),
                    ))
                    .execute(c)?;
                Ok(())
            })
            .await
            .expect("Unexpected panic of a background DB thread")
    }

    pub async fn set_downloaded(&self, req_id: uuid::Uuid, file_path: &Path) -> Result<()> {
        let file_path = file_path.as_os_str().to_owned(); // Need a copy since interact runs on a separate thread
        // and requires 'static.

        let connection = self.pool.get().await?;
        connection
            .interact(move |c| {
                use schema::videos::dsl;
                diesel::update(dsl::videos.find(req_id.to_string()))
                    .set((
                        dsl::download_status.eq(models::DOWNLOAD_STATUS_DOWNLOADED),
                        dsl::downloaded_size.eq(dsl::file_size),
                        dsl::message.eq(""),
                        dsl::file_path.eq(file_path.as_encoded_bytes()),
                    ))
                    .execute(c)?;
                Ok(())
            })
            .await
            .expect("Unexpected panic of a background DB thread")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use core::str::FromStr;
    use googletest::prelude::*;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    #[tokio::test]
    #[googletest::test]
    async fn test_open_db() -> googletest::Result<()> {
        let _db = Database::open(":memory:", Duration::from_millis(1000))
            .await
            .or_fail()?;
        Ok(())
    }

    #[tokio::test]
    #[googletest::test]
    async fn test_insert_and_get_video() -> googletest::Result<()> {
        let uuid = uuid::Uuid::from_str("bf978778-1c5d-44b3-b2c1-1cc253563799").or_fail()?;

        let db = Database::open(":memory:", Duration::from_millis(1000))
            .await
            .or_fail()?;
        db.apply_pending_migrations().await.or_fail()?;
        db.insert_video(uuid, "my video", 1234567).await.or_fail()?;

        let video = db.find_video(uuid).await.or_fail()?;
        expect_that!(
            video,
            eq(&Video {
                id: uuid,
                name: "my video".to_string(),
                file_size: 1234567,
                download_status: DownloadStatus::Pending,
                view_count: 0
            })
        );
        Ok(())
    }

    #[tokio::test]
    #[googletest::test]
    async fn test_increment_view_count() -> googletest::Result<()> {
        let uuid = uuid::Uuid::from_str("bf978778-1c5d-44b3-b2c1-1cc253563799").or_fail()?;

        // Using an in-memory database creates a new db for every connection. That's why we cannot
        // use it if we perform multiple concurrent async operations on the DB.
        // For this reason, we rely on a temporary file.
        let tempfile = NamedTempFile::new().or_fail()?;
        let tempfile = tempfile.path().to_str().or_fail()?;
        let db = Database::open(tempfile, Duration::from_millis(1000))
            .await
            .or_fail()?;
        db.apply_pending_migrations().await.or_fail()?;
        db.insert_video(uuid, "my video", 1234567).await.or_fail()?;

        let incr_a = db.increment_view_count(uuid);
        let incr_b = db.increment_view_count(uuid);
        let incr_c = db.increment_view_count(uuid);

        let (res_a, res_b, res_c) = tokio::join!(incr_a, incr_b, incr_c);
        res_a.or_fail()?;
        res_b.or_fail()?;
        res_c.or_fail()?;

        let video = db.find_video(uuid).await.or_fail()?;
        expect_that!(
            video,
            eq(&Video {
                id: uuid,
                name: "my video".to_string(),
                file_size: 1234567,
                download_status: DownloadStatus::Pending,
                view_count: 3
            })
        );
        Ok(())
    }

    #[tokio::test]
    #[googletest::test]
    async fn test_download_progress() -> googletest::Result<()> {
        let uuid = uuid::Uuid::from_str("bf978778-1c5d-44b3-b2c1-1cc253563799").or_fail()?;

        let db = Database::open(":memory:", Duration::from_millis(1000))
            .await
            .or_fail()?;
        db.apply_pending_migrations().await.or_fail()?;
        db.insert_video(uuid, "my video", 1234567).await.or_fail()?;

        db.update_download_progress(uuid, 1234000).await?;

        let video = db.find_video(uuid).await.or_fail()?;
        expect_that!(
            video,
            eq(&Video {
                id: uuid,
                name: "my video".to_string(),
                file_size: 1234567,
                download_status: DownloadStatus::InProgress((1234000, 1234567)),
                view_count: 0
            })
        );

        db.update_download_progress(uuid, 1234400).await?;

        let video = db.find_video(uuid).await.or_fail()?;
        expect_that!(
            video,
            eq(&Video {
                id: uuid,
                name: "my video".to_string(),
                file_size: 1234567,
                download_status: DownloadStatus::InProgress((1234400, 1234567)),
                view_count: 0
            })
        );
        Ok(())
    }

    #[tokio::test]
    #[googletest::test]
    async fn test_downloaded() -> googletest::Result<()> {
        let uuid = uuid::Uuid::from_str("bf978778-1c5d-44b3-b2c1-1cc253563799").or_fail()?;

        let db = Database::open(":memory:", Duration::from_millis(1000))
            .await
            .or_fail()?;
        db.apply_pending_migrations().await.or_fail()?;
        db.insert_video(uuid, "my video", 1234567).await.or_fail()?;

        let pathbuf: PathBuf = "/path/to/the/file.mp4".into();
        db.set_downloaded(uuid, &pathbuf).await?;

        let video = db.find_video(uuid).await.or_fail()?;
        expect_that!(
            video,
            eq(&Video {
                id: uuid,
                name: "my video".to_string(),
                file_size: 1234567,
                download_status: DownloadStatus::Downloaded("/path/to/the/file.mp4".into()),
                view_count: 0
            })
        );

        Ok(())
    }

    #[tokio::test]
    #[googletest::test]
    async fn test_download_failed() -> googletest::Result<()> {
        let uuid = uuid::Uuid::from_str("bf978778-1c5d-44b3-b2c1-1cc253563799").or_fail()?;

        let db = Database::open(":memory:", Duration::from_millis(1000))
            .await
            .or_fail()?;
        db.apply_pending_migrations().await.or_fail()?;
        db.insert_video(uuid, "my video", 1234567).await.or_fail()?;

        db.set_download_failed(
            uuid,
            "Something failed, but I kid you not, I don't know what it is",
        )
        .await?;

        let video = db.find_video(uuid).await.or_fail()?;
        expect_that!(
            video,
            eq(&Video {
                id: uuid,
                name: "my video".to_string(),
                file_size: 1234567,
                download_status: DownloadStatus::Failed(
                    "Something failed, but I kid you not, I don't know what it is".to_string()
                ),
                view_count: 0
            })
        );

        Ok(())
    }
}
