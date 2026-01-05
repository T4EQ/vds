// @generated automatically by Diesel CLI.

diesel::table! {
    videos (id) {
        id -> Text,
        name -> Text,
        file_size -> BigInt,
        downloaded_size -> BigInt,
        download_status -> BigInt,
        view_count -> BigInt,
        message -> Text,
    }
}
