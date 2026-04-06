use crate::completed::CompletedPage;
use crate::leap_config::LeapConfigPage;
use crate::network_config::NetworkConfigPage;
use crate::storage_config::StorageConfigPage;

use gloo_net::http::Request;
use leap_api::types::ProvisionStatus;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Debug, Clone, PartialEq, Routable)]
pub enum Route {
    #[at("/")]
    Start,
    #[at("/storage")]
    StorageConfig,
    #[at("/network")]
    NetworkConfig,
    #[at("/leap-config")]
    LeapConfig,
    #[at("/completed")]
    Completed,
}

impl From<ProvisionStatus> for Route {
    fn from(value: ProvisionStatus) -> Self {
        match value {
            ProvisionStatus::StorageConfig => Route::StorageConfig,
            ProvisionStatus::NetworkConfig => Route::NetworkConfig,
            ProvisionStatus::LeapConfig => Route::LeapConfig,
            ProvisionStatus::Completed => Route::Completed,
        }
    }
}

#[hook]
pub fn use_provision_redirect(current: Route) {
    let navigator = use_navigator().unwrap();
    use_effect_with((), move |_| {
        spawn_local(async move {
            let response = match Request::get("/provision/status").send().await {
                Ok(r) => r,
                Err(e) => {
                    log::error!("Failed to fetch provision status: {e:?}");
                    return;
                }
            };
            match response.json::<ProvisionStatus>().await {
                Ok(status) => {
                    let target = Route::from(status);
                    if target != current {
                        navigator.replace(&target);
                    }
                }
                Err(e) => {
                    log::error!("Failed to parse provision status: {e:?}");
                }
            }
        });
        || ()
    });
}

#[function_component(StartPage)]
pub fn start_page() -> Html {
    use_provision_redirect(Route::StorageConfig);

    let navigator = use_navigator().unwrap();
    let on_start = Callback::from(move |_| navigator.replace(&Route::StorageConfig));

    html! {
        <div class="start-page">
            <h1>{ "Welcome to LEAP" }</h1>
            <p>{ "Low-Bandwidth Educational Access Platform" }</p>
            <button class="btn-primary" onclick={on_start}>{ "Start" }</button>
        </div>
    }
}

fn switch(route: Route) -> Html {
    match route {
        Route::Start => html! { <StartPage /> },
        Route::StorageConfig => html! { <StorageConfigPage /> },
        Route::NetworkConfig => html! { <NetworkConfigPage /> },
        Route::LeapConfig => html! { <LeapConfigPage /> },
        Route::Completed => html! { <CompletedPage /> },
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
