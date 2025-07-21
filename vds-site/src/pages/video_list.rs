use gloo_net::http::Request;
use vds_api::api::content::local::get::{Response, Video};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use super::player::VideoPlayer;

#[derive(yew::Properties, PartialEq, Eq)]
pub struct VideoCardProps {
    index: usize,
    video: Video,
}

#[function_component(VideoCard)]
pub fn video_card(VideoCardProps { index, video }: &VideoCardProps) -> Html {
    let byte_to_megabytes = |bytes: usize| bytes as f64 / 1024.0 / 1024.0;

    let show_video = use_state(|| false);

    let action = {
        let show_video = show_video.clone();
        move |_| {
            show_video.set(true);
        }
    };

    if *show_video {
        html! {
            <VideoPlayer id={video.id.clone()}/>
        }
    } else {
        html! {
            <div class="video-card" key={*index}>
                <div class="video-icon">
                    <span class="play-icon">{ "▶" }</span>
                </div>
                <div class="video-info">
                    <h3 class="video-title">{ &video.name }</h3>
                    <div class="video-meta">
                        <span class="video-size">{ byte_to_megabytes(video.size) } {" MB"}</span>
                        <span class="video-id">{"ID: "} { &video.id }</span>
                    </div>
                </div>
                <button class="play-button" onclick={action}>{ "Play" }</button>
            </div>
        }
    }
}

async fn fetch_video_content() -> Option<Vec<Video>> {
    let response = match Request::get("/api/content/local").send().await {
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
        Response::Collection(video_list) => Some(video_list),
        Response::Single(_) => {
            log::warn!("Unexpected single video response for collection request");
            None
        }
    }
}

#[function_component(VideoList)]
pub fn video_list() -> Html {
    let videos: UseStateHandle<Option<Vec<Video>>> = use_state(|| None);

    use_effect_with(videos.clone(), move |vids| {
        if vids.is_none() {
            let vids = vids.clone();
            spawn_local(async move {
                vids.set(fetch_video_content().await);
            });
        }
        || ()
    });

    if let Some(videos) = &*videos {
        html! {
            <div class="dashboard">
                <header class="dashboard-header">
                    <h1>{ "Video Library" }</h1>
                    <p>{ "Locally Available Educational Content" }</p>
                </header>

                <div class="video-grid">
                    { for (*videos).iter().enumerate().map(|(index, video)| html! {
                        <VideoCard video={video.clone()} index={index} />
                    }) }
                </div>
            </div>
        }
    } else {
        html! {
            <div class="dashboard">
                <header class="dashboard-header">
                    <h1>{ "Video Library" }</h1>
                    <p>{ "Loading..." }</p>
                </header>
            </div>
        }
    }
}
