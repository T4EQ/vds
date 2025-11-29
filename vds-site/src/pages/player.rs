use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use vds_api::api::content::meta::id::get::{LocalVideoMeta, Response};

async fn fetch_video_content(id: &str) -> Option<LocalVideoMeta> {
    let response = Request::get(&format!("/api/content/meta/{id}"))
        .send()
        .await
        .inspect_err(|e| {
            log::error!("Failed to fetch videos. Error performing HTTP request: {e:?}");
        })
        .ok()?;

    let response = response
        .json::<Response>()
        .await
        .inspect_err(|e| {
            log::error!("Failed to fetch videos. Error decoding json: {e:?}");
        })
        .ok()?;

    response.meta
}

#[derive(yew::Properties, PartialEq, Eq)]
pub struct VideoPlayerProps {
    pub id: String,
}

#[function_component(VideoPlayer)]
pub fn video_player(VideoPlayerProps { id }: &VideoPlayerProps) -> Html {
    let video: UseStateHandle<Option<LocalVideoMeta>> = use_state(|| None);

    use_effect_with((id.to_string(), video.clone()), move |(id, vid)| {
        if vid.is_none() {
            let vid = vid.clone();
            let id = id.clone();
            spawn_local(async move {
                vid.set(fetch_video_content(&id).await);
            });
        }
        || ()
    });

    let name = video
        .as_ref()
        .map(|v| &v.name as &str)
        .unwrap_or("Loading...");

    let path = format!("/api/content/{id}");
    html! {
        <div class="dashboard">
            <header class="dashboard-header">
                <h1>{name}</h1>
            </header>

            <video controls=true autoplay=true class="video-player">
                <source src={path} type="video/mp4" />
            </video>
        </div>
    }
}
