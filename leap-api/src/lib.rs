//! The `leap-api` crate defines common data types shared by `leap-site` and `leap-server`.
//!
//! The create follows these conventions:
//! - Each API endpoint of the `leap-server` defines a full namespace path.
//! - For each endpoint namespace, `leap-api` defines a nested namespace
//!   with the API method of the endpoint.
//! - Inside the namespace for a given API endpoint, the following types are defined:
//!   - If the request method is `GET`, a `Query` type may be defined to indicate what query
//!     parameters can be sent to the server.
//!   - If the endpoint returns a JSON body, a `Response` type defines its contents.
//!   - Any additional types required to define either the query or the response.
//!
//! The supported endpoints are:
//!  - `POST` `api/manifest/fetch`. Triggers an immediate fetch of the manifest, causing the LEAP to
//!    update its cached content.
//!  - `GET` `api/manifest/latest`. Returns the latest manifest that is in use by the LEAP.
//!  - `GET` `api/content/meta`. Returns a list of the content metadata in the local server (LEAP).
//!  - `GET` `api/content/meta/{id}`. Returns the metadata of the requested id.
//!  - `GET` `api/content/{id}`. Obtains the requested content from the server. The path indicates
//!    the resource ID.

mod types;

pub mod api {
    pub mod version {
        pub mod get {
            pub use crate::types::BuildInfo;

            /// The response to the `GET` `api/version` request
            pub type Response = BuildInfo;
        }
    }

    pub mod content {
        pub mod meta {
            pub mod get {
                pub use crate::types::{GroupedSection, LocalVideoMeta, Progress, VideoStatus};

                /// The response to the `GET` `api/content/meta` request
                #[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq)]
                pub struct Response {
                    pub videos: Vec<GroupedSection>,
                }
            }

            pub mod id {
                pub mod get {
                    pub use crate::types::{LocalVideoMeta, Progress, VideoStatus};

                    /// The response to the `GET` `api/content/meta/{id}` request
                    #[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq)]
                    pub struct Response {
                        pub meta: Option<LocalVideoMeta>,
                    }
                }
            }
        }
    }
}
