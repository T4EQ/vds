// TODO: make this more organized instead of copying the data types.
// TODO: add type safety

pub mod api {

    pub mod content {

        pub mod remote {
            pub mod get {
                #[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq, Clone)]
                pub struct Video {
                    pub id: String,
                    pub name: String,
                    pub size: usize,
                    pub local: bool,
                }

                #[derive(Default, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
                pub struct Response {
                    pub videos: Vec<Video>,
                }

                #[derive(Default, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
                pub struct Query {
                    pub limit: Option<usize>,
                    // TODO: add pagination (offset).
                }
            }
        }

        pub mod local {
            pub mod get {
                #[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq, Clone)]
                pub enum VideoStatus {
                    Downloading,
                    Downloaded,
                    Failed,
                }

                #[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq, Clone)]
                pub struct Video {
                    pub id: String,
                    pub name: String,
                    pub size: usize,
                    pub status: VideoStatus,
                }

                #[derive(Default, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
                pub struct Query {
                    pub id: Option<String>,
                    /// Maximum number of results to return
                    pub limit: Option<usize>,
                    // TODO: add pagination (offset).
                }

                #[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
                pub enum Response {
                    Single(Option<Video>),
                    Collection(Vec<Video>),
                }
            }

            pub mod delete {
                #[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
                pub struct Query {
                    pub id: String,
                }
            }

            pub mod put {
                #[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
                pub struct Query {
                    pub id: String,
                }
            }
        }

        #[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
        pub struct Query {
            pub id: String,
        }
    }
}
