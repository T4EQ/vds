use vds_api::api::content::local::get::{Video, VideoStatus};
use yew::prelude::*;

#[derive(yew::Properties, PartialEq, Eq)]
pub struct VideoCardProps {
    index: usize,
    video: Video,
}

#[function_component(VideoCard)]
pub fn video_card(VideoCardProps { index, video }: &VideoCardProps) -> Html {
    let byte_to_megabytes = |bytes: usize| bytes as f64 / 1024.0 / 1024.0;

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
            <button class="play-button">{ "Play" }</button>
        </div>
    }
}

#[function_component(VideoList)]
pub fn video_list() -> Html {
    let videos: Vec<Video> = vec![
        Video {
            id: "1".to_string(),
            name: "Introduction to Mathematics".to_string(),
            size: 245 * 1024 * 1024,
            status: VideoStatus::Downloaded,
        },
        Video {
            id: "2".to_string(),
            name: "Basic Science Concepts".to_string(),
            size: 312 * 1024 * 1024,
            status: VideoStatus::Downloaded,
        },
        Video {
            id: "3".to_string(),
            name: "English Grammar Fundamentals".to_string(),
            size: 189 * 1024 * 1024,
            status: VideoStatus::Downloaded,
        },
        Video {
            id: "4".to_string(),
            name: "History of Ancient Civilizations".to_string(),
            size: 456 * 1024 * 1024,
            status: VideoStatus::Downloaded,
        },
        Video {
            id: "5".to_string(),
            name: "Environmental Science Basics".to_string(),
            size: 378 * 1024 * 1024,
            status: VideoStatus::Downloaded,
        },
    ];

    html! {
        <div class="dashboard">
            <header class="dashboard-header">
                <h1>{ "Video Library" }</h1>
                <p>{ "Locally Available Educational Content" }</p>
            </header>

            <div class="video-grid">
                { for videos.into_iter().enumerate().map(|(index, video)| html! {
                    <VideoCard video={video} index={index} />
                }) }
            </div>
        </div>
    }
}
