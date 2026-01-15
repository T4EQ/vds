use yew::prelude::*;
use yew_router::prelude::*;

use crate::pages::player::VideoPlayer;
use crate::pages::video_list::VideoList;

#[derive(Debug, Clone, PartialEq, Routable)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/player/:id")]
    Player { id: String },
}

fn switch(route: Route) -> Html {
    match route {
        Route::Home => {
            html! {
                <VideoList>
                </VideoList>
            }
        }
        Route::Player { id } => {
            html! {
                <VideoPlayer id={id}>
                </VideoPlayer>
            }
        }
    }
}

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}
