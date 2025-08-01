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

mod types;

pub mod api {
    pub mod content {
        pub mod remote {
            pub mod get {
                pub use crate::types::RemoteVideoMeta;

                /// The response to the `GET` `api/content/remote` request
                #[derive(Default, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
                pub struct Response {
                    /// List of videos of the remote
                    pub videos: Vec<RemoteVideoMeta>,
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
            pub mod single {
                pub mod get {
                    pub use crate::types::{LocalVideoMeta, Progress, VideoStatus};

                    /// The query that can be used in a `GET` `api/content/local/single` request
                    #[derive(
                        Default, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq,
                    )]
                    pub struct Query {
                        /// Unique identifier of the video
                        pub id: String,
                        /// Maximum number of videos to list
                        pub limit: Option<usize>,
                        // TODO: add pagination (offset).
                    }

                    /// The response to the `GET` `api/content/local/single` request
                    #[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq)]
                    pub struct Response {
                        /// Video metadata, if found
                        pub video: Option<LocalVideoMeta>,
                    }
                }
            }

            pub mod get {
                pub use crate::types::{LocalVideoMeta, Progress, VideoStatus};

                /// The query that can be used in a `GET` `api/content/local` request
                #[derive(Default, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
                pub struct Query {
                    /// Maximum number of videos to list
                    pub limit: Option<usize>,
                    // TODO: add pagination (offset).
                }

                /// The response to the `GET` `api/content/local` request
                #[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq)]
                pub struct Response {
                    /// Locally cached content
                    pub videos: Vec<LocalVideoMeta>,
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
