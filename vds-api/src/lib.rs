// TODO: make this more organized instead of copying the data types.
// TODO: add type safety

//! The `vds-api` crate defines common data types shared by `vds-site` and `vds-server`.
//!
//! The create follows these conventions:
//! - Each API endpoint of the `vds-server` defines a full namespace path.
//! - For each endpoint namespace, `vds-api` defines a nested namespace
//!   with the API method of the endpoint.
//! - Inside the namespace for a given API endpoint, the following types are defined:
//!   - If the request method is `GET`, a `Query` type may be defined to indicate what query
//!     parameters can be sent to the server.
//!   - If the endpoint returns a JSON body, a `Response` type defines its contents.
//!   - Any additional types required to define either the query or the response.
//!
//! The following terminology is used in the VDS:
//! - `local` refers to content local to the VDS server.
//! - `remote` refers to content in the remote servers (e.g.: S3, not in the LAN).
//!
//! The supported endpoints are:
//!  - `GET` `api/content/remote`. Returns a list of the content in the remote server (e.g.: S3).
//!  - `GET` `api/content/local`. Returns a list of the content in the local server (VDS).
//!  - `DELETE` `api/content/local`. Deletes a piece of content from the local server.
//!  - `PUT` `api/content/local`. Caches a piece of content from the remote server in the local
//!    server.
//!  - `GET` `api/content`. Obtains the requested content from the server. The Query requires an
//!    resource ID
//!  - `GET` `api/content/{id}`. Obtains the requested content from the server. The path indicates
//!    the resource ID.

pub mod api {
    pub mod content {
        pub mod remote {
            pub mod get {

                /// Metadata of a single video present in the remote server.
                #[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq, Clone)]
                pub struct Video {
                    /// Unique identifier of the video
                    pub id: String,
                    /// Human-readable name of the video
                    pub name: String,
                    /// Size of the video in bytes
                    pub size: usize,
                    /// flag indicating whether the video is also locally cached.
                    pub local: bool,
                }

                /// The response to the `GET` `api/content/remote` request
                #[derive(Default, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
                pub struct Response {
                    /// List of videos of the remote
                    pub videos: Vec<Video>,
                }

                /// The query that can be used in a `GET` `api/content/remote` request
                #[derive(Default, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
                pub struct Query {
                    /// Maximum number of videos to list
                    pub limit: Option<usize>,
                    // TODO: add pagination (offset).
                }
            }
        }

        pub mod local {
            pub mod get {
                /// Download progress. A number from 0 to 1, where 1 indicates completed and 0 not
                /// started.
                #[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Clone)]
                pub struct Progress(pub f64);

                /// The status of the video download
                #[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Clone)]
                pub enum VideoStatus {
                    /// The video download is in progress
                    Downloading(Progress),
                    /// The video download is completed
                    Downloaded,
                    /// The video download failed
                    Failed,
                }

                /// Metadata of a single video of the local server.
                #[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Clone)]
                pub struct Video {
                    /// Unique identifier of the video
                    pub id: String,
                    /// Human-readable name of the video
                    pub name: String,
                    /// Size of the video in bytes
                    pub size: usize,
                    /// Download status
                    pub status: VideoStatus,
                }

                /// The query that can be used in a `GET` `api/content/local` request
                #[derive(Default, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
                pub struct Query {
                    /// Optionally, we can ask for a single result, if we know the ID.
                    pub id: Option<String>,
                    /// Maximum number of videos to list
                    pub limit: Option<usize>,
                    // TODO: add pagination (offset).
                }

                /// The response to the `GET` `api/content/local` request
                #[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq)]
                pub enum Response {
                    /// Response to a request to list a specific video (if `id` was given in the request).
                    Single(Option<Video>),
                    /// Response to a request to list all videos (if `id` was not given in the request).
                    Collection(Vec<Video>),
                }
            }

            pub mod delete {
                /// The query that can be used in a `DELETE` `api/content/local` request
                #[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
                pub struct Query {
                    /// Unique identifier of the video
                    pub id: String,
                }
            }

            pub mod put {
                /// The query that can be used in a `PUT` `api/content/local` request
                #[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
                pub struct Query {
                    /// Unique identifier of the video
                    pub id: String,
                }
            }
        }

        pub mod get {
            /// The query that can be used in a `GET` `api/content` request
            #[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
            pub struct Query {
                /// Unique identifier of the video
                pub id: String,
            }
        }
    }
}
