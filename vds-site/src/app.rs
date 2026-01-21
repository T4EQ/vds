use yew::prelude::*;
use yew_router::prelude::*;

use crate::context::ContentProvider;
use crate::pages::dashboard::Dashboard;
use crate::pages::player::VideoPlayer;
use crate::pages::status::StatusDashboard;

#[derive(Debug, Clone, PartialEq, Routable)]
pub enum Route {
    #[at("/")]
    Home,

    #[at("/playlists/:playlist_id")]
    Playlist { playlist_id: usize },

    #[at("/playlists/:playlist_id/videos/:video_id")]
    Video {
        playlist_id: usize,
        video_id: String,
    },

    #[at("/status")]
    Status,
}

fn switch(route: Route) -> Html {
    match route {
        Route::Home => {
            html! {
                <Dashboard>
                </Dashboard>
            }
        }
        Route::Playlist { playlist_id } => {
            html! {
                <VideoPlayer playlist_id={playlist_id} video_id={None as Option<String>}>
                </VideoPlayer>
            }
        }
        Route::Video {
            playlist_id,
            video_id,
        } => {
            html! {
                <VideoPlayer playlist_id={playlist_id} video_id={Some(video_id)}>
                </VideoPlayer>
            }
        }
        Route::Status => {
            html! {
                <StatusDashboard>
                </StatusDashboard>
            }
        }
    }
}

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <ContentProvider>
            <BrowserRouter>
                <Switch<Route> render={switch} />
            </BrowserRouter>
        </ContentProvider>
    }
}
