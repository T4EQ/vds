CREATE TABLE videos (
    id VARCHAR NOT NULL PRIMARY KEY,
    name VARCHAR NOT NULL,
    file_size BIG INT NOT NULL,
    downloaded_size BIG INT NOT NULL DEFAULT 0,
    download_status BIG INT NOT NULL DEFAULT 0,
    view_count BIG INT NOT NULL DEFAULT 0,
    message VARCHAR NOT NULL DEFAULT '',

    -- file_path could be a VARCHAR. However, because Rust uses OsString to represent PathBuf
    -- constructing a String from Path or PathBuf is a fallible operation, because there are no
    -- guarantees that the OsString behind the Path is actually UTF-8 encoded.
    -- Therefore, a BLOB translates to Vec<u8>, which can be translated to/from PathBuf easily
    -- in an infallible way.
    file_path BLOB NOT NULL DEFAULT ''
)
