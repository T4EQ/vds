use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(StartPage)]
pub fn start_page() -> Html {
    html! {
        <h1>
            { "Welcome to LEAP" }
        </h1>
    }
}

#[derive(Debug, Clone, PartialEq, Routable)]
pub enum Route {
    #[at("/")]
    Start,
}

fn switch(route: Route) -> Html {
    match route {
        Route::Start => {
            html! {
                <StartPage>
                </StartPage>
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
