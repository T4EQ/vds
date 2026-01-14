use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

use vds_api::api::content::meta::get::{GroupedSection, LocalVideoMeta, Response, VideoStatus};

use crate::app::Route;

#[derive(yew::Properties, PartialEq)]
pub struct VideoCardProps {
    section_name: String,
    video: LocalVideoMeta,
}

#[function_component(VideoCard)]
pub fn video_card(VideoCardProps { video, .. }: &VideoCardProps) -> Html {
    let byte_to_megabytes = |bytes: usize| bytes as f64 / 1024.0 / 1024.0;

    let downloaded = video.status == VideoStatus::Downloaded;
    let play_button = if downloaded {
        let destination = Route::Player {
            id: video.id.clone(),
        };
        html! {
            <Link<Route> to={destination} classes={"play-button"}>
                { "Play" }
            </Link<Route>>
        }
    } else {
        html! {
            <button disabled=true class={classes!{"play-button"}}>
                { "Play" }
            </button>
        }
    };

    html! {
        <div class="video-card">
            <div class={classes!{"video-icon", (!downloaded).then_some("disabled")}}>
                <span class="play-icon">{ "▶" }</span>
            </div>
            <div class="video-info">
                <h3 class="video-title">{ &video.name }</h3>
                <div class="video-meta">
                    <span class="video-size">{ byte_to_megabytes(video.size) } {" MB"}</span>
                    <span class="video-id">{"ID: "} { &video.id }</span>
                </div>
            </div>
            { play_button }
        </div>
    }
}

async fn fetch_video_content() -> Option<Vec<GroupedSection>> {
    let response = match Request::get("/api/content/meta").send().await {
        Ok(v) => v,
        Err(e) => {
            log::error!("Failed to fetch content meta. Error performing HTTP request: {e:?}");
            return None;
        }
    };

    let response = match response.json::<Response>().await {
        Ok(v) => v,
        Err(e) => {
            log::error!("Failed to fetch videos. Error decoding json: {e:?}");
            return None;
        }
    };

    Some(response.videos)
}

#[function_component(VideoList)]
pub fn video_list() -> Html {
    let sections: UseStateHandle<Option<Vec<GroupedSection>>> = use_state(|| None);

    use_effect_with(sections.clone(), move |sections| {
        if sections.is_none() {
            let sections = sections.clone();
            spawn_local(async move {
                sections.set(fetch_video_content().await);
            });
        }
        || ()
    });

    if let Some(sections) = &*sections {
        html! {
            <div class="dashboard">
                <header class="dashboard-header">
                    <h1>{ "Video Library" }</h1>
                    <p>{ "Locally Available Educational Content" }</p>
                </header>

                <div class="video-grid">
                    {
                        sections.iter().flat_map(|section|
                            // TODO: Split videos by section. At this time I do not want to change
                            // the UI, but it needs to be done.
                            section.content.iter().map(|video|
                                html! { <VideoCard section_name={section.name.clone()} video={video.clone()} /> })
                        ).collect::<Html>()
                    }
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
