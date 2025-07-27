use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use vds_api::api::content::local::get::{Response, Video};

async fn fetch_video_content(id: &str) -> Option<Video> {
    let response = match Request::get("/api/content/local")
        .query([("id", id)])
        .send()
        .await
    {
        Ok(v) => v,
        Err(e) => {
            log::error!("Failed to fetch videos. Error performing HTTP request: {e:?}");
            return None;
        }
    };

    let response_json = match response.json::<Response>().await {
        Ok(v) => v,
        Err(e) => {
            log::error!("Failed to fetch videos. Error decoding json: {e:?}");
            return None;
        }
    };

    match response_json {
        Response::Single(video) => video,
        Response::Collection(_) => {
            log::warn!("Unexpected collection response for single request");
            None
        }
    }
}

#[derive(yew::Properties, PartialEq, Eq)]
pub struct VideoPlayerProps {
    pub id: String,
}

#[function_component(VideoPlayer)]
pub fn video_player(VideoPlayerProps { id }: &VideoPlayerProps) -> Html {
    let video: UseStateHandle<Option<Video>> = use_state(|| None);

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
