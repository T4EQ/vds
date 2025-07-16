use yew::prelude::*;

#[derive(Clone, PartialEq)]
struct Video {
    id: u32,
    name: String,
    size: String,
    download_date: String,
}

#[function_component(VideoList)]
fn video_list() -> Html {
    let videos = vec![
        Video {
            id: 1,
            name: "Introduction to Mathematics".to_string(),
            size: "245 MB".to_string(),
            download_date: "2025-07-10".to_string(),
        },
        Video {
            id: 2,
            name: "Basic Science Concepts".to_string(),
            size: "312 MB".to_string(),
            download_date: "2025-07-09".to_string(),
        },
        Video {
            id: 3,
            name: "English Grammar Fundamentals".to_string(),
            size: "189 MB".to_string(),
            download_date: "2025-07-08".to_string(),
        },
        Video {
            id: 4,
            name: "History of Ancient Civilizations".to_string(),
            size: "456 MB".to_string(),
            download_date: "2025-07-07".to_string(),
        },
        Video {
            id: 5,
            name: "Environmental Science Basics".to_string(),
            size: "378 MB".to_string(),
            download_date: "2025-07-06".to_string(),
        },
    ];

    html! {
        <div class="dashboard">
            <header class="dashboard-header">
                <h1>{ "Video Library" }</h1>
                <p>{ "Locally Available Educational Content" }</p>
            </header>

            <div class="video-grid">
                { for videos.iter().map(|video| html! {
                    <div class="video-card" key={video.id}>
                        <div class="video-icon">
                            <span class="play-icon">{ "▶" }</span>
                        </div>
                        <div class="video-info">
                            <h3 class="video-title">{ &video.name }</h3>
                            <div class="video-meta">
                                <span class="video-size">{ &video.size }</span>
                                <span class="video-date">{ format!("Downloaded: {}", &video.download_date) }</span>
                            </div>
                        </div>
                        <button class="play-button">{ "Play" }</button>
                    </div>
                }) }
            </div>
        </div>
    }
}

#[function_component(LoginForm)]
fn login_form(props: &LoginFormProps) -> Html {
    let on_username_change = {
        let username = props.username.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            username.set(input.value());
        })
    };

    let on_password_change = {
        let password = props.password.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            password.set(input.value());
        })
    };

    let on_submit = {
        let on_login = props.on_login.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            on_login.emit(());
        })
    };

    html! {
        <main class="login-container">
            <div class="login-card">
                <h1 class="login-title">{ "VDS Login" }</h1>
                <p class="login-subtitle">{ "Video Delivery System" }</p>

                <form class="login-form" onsubmit={on_submit}>
                    <div class="form-group">
                        <label for="username">{ "Username" }</label>
                        <input
                            type="text"
                            id="username"
                            name="username"
                            placeholder="Enter your username"
                            value={(*props.username).clone()}
                            onchange={on_username_change}
                            required=true
                        />
                    </div>

                    <div class="form-group">
                        <label for="password">{ "Password" }</label>
                        <input
                            type="password"
                            id="password"
                            name="password"
                            placeholder="Enter your password"
                            value={(*props.password).clone()}
                            onchange={on_password_change}
                            required=true
                        />
                    </div>

                    <button type="submit" class="login-button">{ "Sign In" }</button>
                </form>

                <div class="login-footer">
                    <p>{ "Offline Video Delivery for Remote Education" }</p>
                </div>
            </div>
        </main>
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Auth {
    auth_success: bool,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct AuthRequest {
    username: String,
    password: String,
}

async fn try_authenticate_async(username: &str, password: &str) -> bool {
    let rsp: Auth = gloo_net::http::Request::new("api/user/auth")
        .method(gloo_net::http::Method::PUT)
        .json(&AuthRequest {
            username: username.to_string(),
            password: password.to_string(),
        })
        .unwrap()
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    // FIXME: return session auth token instead
    log::info!(
        "Auth for username = {username}, password = {password} is {}",
        rsp.auth_success
    );
    rsp.auth_success
}

#[derive(Properties, PartialEq)]
struct LoginFormProps {
    username: UseStateHandle<String>,
    password: UseStateHandle<String>,
    on_login: Callback<()>,
}

#[function_component(App)]
pub fn app() -> Html {
    let username = use_state(String::new);
    let password = use_state(String::new);
    let is_authenticated = use_state(|| false);

    let on_login = {
        let username = username.clone();
        let password = password.clone();
        let is_authenticated = is_authenticated.clone();

        Callback::from(move |_| {
            let username = username.clone();
            let password = password.clone();
            let is_authenticated = is_authenticated.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if try_authenticate_async(&username, &password).await {
                    is_authenticated.set(true);
                }
            });
        })
    };

    log::info!("Rendering App component: {is_authenticated:?}");

    if *is_authenticated {
        html! { <VideoList /> }
    } else {
        html! { <LoginForm username={username} password={password} on_login={on_login} /> }
    }
}
