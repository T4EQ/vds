use std::hash::{DefaultHasher, Hasher};
use yew::prelude::*;
use yew_router::prelude::*;

use crate::context::ContentContextHandle;

#[derive(yew::Properties, PartialEq)]
pub struct PlaylistCardProps {
    pub playlist_name: String,
    pub num_videos: usize,
    pub first_video_id: Option<String>,
}

#[function_component(PlaylistCard)]
pub fn playlist_card(
    PlaylistCardProps {
        playlist_name,
        num_videos,
        first_video_id,
    }: &PlaylistCardProps,
) -> Html {
    let navigator = use_navigator();

    let onclick = if *num_videos > 0 {
        let first_video_id = first_video_id.clone();
        Callback::from(move |_| {
            if let Some(navigator) = &navigator
                && let Some(id) = &first_video_id
            {
                navigator.push(&crate::app::Route::Player { id: id.clone() });
            }
        })
    } else {
        Callback::noop()
    };

    let first_letter = playlist_name.chars().next().unwrap_or_default();
    let mut hasher = DefaultHasher::new();
    hasher.write(playlist_name.as_bytes());
    let hue = hasher.finish() % 360;
    let icon_style = format!("color: hsl({}, 70%, 60%);", hue);

    html! {
        <div {onclick} class={classes!("card", (*num_videos == 0).then_some("unavailable"))}>
            <div class="icon" style={icon_style}>{ first_letter }</div>
            <div class="details">
                <h3>{ playlist_name }</h3>
                <span>{ format!("{} videos", num_videos) }</span>
            </div>
            <div class="arrow"> { "\u{203A}" }</div>
        </div>
    }
}

#[function_component(PlaylistsList)]
pub fn playlists_list() -> Html {
    let context = use_context::<ContentContextHandle>().expect("ContentContext not found");

    let Some(sections) = &context.sections else {
        return html! {
            <p>{"Loading..."}</p>
        };
    };

    if sections.is_empty() {
        html! {
            <p1>{ "No playlists available yet." }</p1>
        }
    } else {
        html! {
                <div class="playlist-list list">
                {
                    sections.iter().map(|section| {
                        let num_videos = section.content.len();
                        let first_video_id = section.content.first().map(|v| v.id.clone());
                        html! { <PlaylistCard playlist_name={section.name.clone()} num_videos={num_videos} first_video_id={first_video_id} /> }
                    }).collect::<Html>()
                }
                </div>
        }
    }
}

#[function_component(Dashboard)]
pub fn dashboard() -> Html {
    html! {
        <div class="page dashboard-page">
            <header class="header">
                <h1>{ "Playlists" }</h1>
            </header>
            <PlaylistsList/>
        </div>
    }
}
