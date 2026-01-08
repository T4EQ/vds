CREATE TABLE videos (
    id VARCHAR NOT NULL PRIMARY KEY,
    name VARCHAR NOT NULL,
    file_size BIG INT NOT NULL,
    downloaded_size BIG INT NOT NULL DEFAULT 0,
    download_status BIG INT NOT NULL DEFAULT 0,
    view_count BIG INT NOT NULL DEFAULT 0,
    message VARCHAR NOT NULL DEFAULT '',
    file_path BLOB NOT NULL DEFAULT ''
)
