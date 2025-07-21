use yew::prelude::*;
use yew_router::prelude::*;

use crate::pages::admin::AdminPage;
use crate::pages::video_list::VideoList;

#[derive(Debug, Clone, Copy, PartialEq, Routable)]
enum Route {
    #[at("/")]
    Home,
    #[at("/admin")]
    Admin,
}

fn switch(route: Route) -> Html {
    match route {
        Route::Home => {
            html! {
                <VideoList>
                </VideoList>
            }
        }
        Route::Admin => {
            html! {
                <AdminPage>
                </AdminPage>
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
