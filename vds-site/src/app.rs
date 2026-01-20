use yew::prelude::*;
use yew_router::prelude::*;

use crate::context::ContentProvider;
use crate::pages::dashboard::Dashboard;
use crate::pages::player::VideoPlayer;

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
                <Dashboard>
                </Dashboard>
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
        <ContentProvider>
            <BrowserRouter>
                <Switch<Route> render={switch} />
            </BrowserRouter>
        </ContentProvider>
    }
}
