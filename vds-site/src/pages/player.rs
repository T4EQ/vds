use crate::context::ContentContextHandle;
use gloo_net::http::Request;
use vds_api::api::content::meta::get::VideoStatus::{Downloaded, Downloading, Failed, Pending};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(yew::Properties, PartialEq, Eq)]
pub struct VideoPlayerProps {
    pub id: String,
}

#[function_component(VideoPlayer)]
pub fn video_player(VideoPlayerProps { id }: &VideoPlayerProps) -> Html {
    let context = use_context::<ContentContextHandle>().expect("ContentContext not found");
    let navigator = use_navigator().expect("Navigator not found");
    let on_back_click_navigator = navigator.clone();
    let on_back_click = Callback::from(move |_| {
        on_back_click_navigator.back();
    });

    {
        let context = context.clone();
        let id = id.clone();
        use_effect_with(id.clone(), move |id| {
            let id = id.clone();
            spawn_local(async move {
                if let Ok(resp) = Request::post(&format!("/api/content/{id}/view"))
                    .send()
                    .await
                {
                    if resp.ok() {
                        if let Some(sections) = context.sections.as_ref() {
                            let mut new_sections = (**sections).clone();
                            let mut found = false;
                            for section in &mut new_sections {
                                for video in &mut section.content {
                                    if video.id == id {
                                        video.view_count += 1;
                                        found = true;
                                        break;
                                    }
                                }
                                if found {
                                    break;
                                }
                            }
                            if found {
                                context.dispatch(new_sections);
                            }
                        }
                    }
                }
            });
            || ()
        });
    }

    let Some(sections) = &context.sections else {
        return html! {
            <div class={"page"}>
                <p>{"Loading..."}</p>
            </div>
        };
    };

    let Some((section, active_video)) = sections
        .iter()
        .find_map(|s| s.content.iter().find(|v| v.id == *id).map(|v| (s, v)))
    else {
        return html! {
            <div class={"page"}>
                <p>{"Invalid video ID."}</p>
           </div>
        };
    };

    let video_path = format!("/api/content/{}", active_video.id);

    html! {
        <div class="page player-page">
            <div class="player-main">
                <header>
                    <button class="back-button" onclick={on_back_click}>
                        <svg xmlns="http://www.w3.org/2000/svg" height="30px" viewBox="0 0 24 24" width="24px" fill="#FFFFFF">
                            <path d="M0 0h24v24H0z" fill="none"/>
                            <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z"/>
                        </svg>
                    </button>
                    <h1>{ &section.name }</h1>
                </header>

                <video key={active_video.id.clone()} controls=true autoplay=true class="video-player">
                    <source src={video_path} type="video/mp4" />
                </video>

                <h2>{ &active_video.name }</h2>

                <div class={"details"}>
                    <span>{ format!("{} views", active_video.view_count) }</span>
                </div>
            </div>

            <div class={"video-list list"}>
            {
                section.content.iter().enumerate().map(|(i, video)| {
                    let is_active = video.id == active_video.id;
                    let icon = if is_active {
                        html! {
                            <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor" xmlns="http://www.w3.org/2000/svg">
                                <rect x="5" y="10" width="2" height="4" rx="1">
                                    <animate attributeName="height" values="4;16;4" begin="0s" dur="1.2s" repeatCount="indefinite" />
                                    <animate attributeName="y" values="10;4;10" begin="0s" dur="1.2s" repeatCount="indefinite" />
                                </rect>
                                <rect x="11" y="10" width="2" height="4" rx="1">
                                    <animate attributeName="height" values="4;16;4" begin="0.2s" dur="1.2s" repeatCount="indefinite" />
                                    <animate attributeName="y" values="10;4;10" begin="0.2s" dur="1.2s" repeatCount="indefinite" />
                                </rect>
                                <rect x="17" y="10" width="2" height="4" rx="1">
                                    <animate attributeName="height" values="4;16;4" begin="0.4s" dur="1.2s" repeatCount="indefinite" />
                                    <animate attributeName="y" values="10;4;10" begin="0.4s" dur="1.2s" repeatCount="indefinite" />
                                </rect>
                            </svg>
                        }
                    } else {
                        html! { <span>{ format!("{:02}", i + 1) }</span> }
                    };

                    let (is_downloaded, status_text) = match &video.status {
                        Downloaded => (true, format!("{} views", video.view_count)),
                        Downloading(progress) => (false, format!("Downloading ({:.0}%)", progress.0 * 100.0)),
                        Pending => (false, "Pending".to_string()),
                        Failed(_) => (false, "Download failed".to_string()),
                    };

                    let onclick = if is_downloaded {
                        let list_item_navigator = navigator.clone();
                        let video_id = video.id.clone();
                        Callback::from(move |_| {
                            list_item_navigator.replace(&crate::app::Route::Player { id: video_id.clone() });
                        })
                    } else {
                        Callback::noop()
                    };

                    html! {
                        <div {onclick} class={classes!("card", is_active.then_some("active"), (!is_downloaded).then_some("unavailable"))}>
                            <div class="icon">{ icon }</div>
                            <div class="details">
                                <h3>{ &video.name }</h3>
                                <span>{ status_text }</span>
                            </div>
                        </div>
                    }
                }).collect::<Html>()
            }
            </div>
        </div>
    }
}
