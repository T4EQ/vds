use crate::app::{Route, use_provision_redirect};
use gloo_net::http::Request;
use leap_api::types::{IpConfig, NetworkConfig, StaticIpConfig, WiredConfig, WirelessConfig};
use secrecy::SecretString;
use std::net::Ipv4Addr;
use std::str::FromStr;
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Copy, Clone, PartialEq)]
enum ConnectionType {
    Wired,
    Wireless,
}

#[derive(Copy, Clone, PartialEq)]
enum IpMode {
    Dhcp,
    Static,
}

#[function_component(NetworkConfigPage)]
pub fn network_config_page() -> Html {
    use_provision_redirect(Route::NetworkConfig);

    let connection_type = use_state(|| ConnectionType::Wired);
    let ip_mode = use_state(|| IpMode::Dhcp);
    let ssid = use_state(String::new);
    let password = use_state(String::new);
    let ip_address = use_state(String::new);
    let gateway = use_state(String::new);
    let net_mask = use_state(String::new);
    let toast: UseStateHandle<Option<String>> = use_state(|| None);
    let submitting = use_state(|| false);

    let on_connection_change = {
        let connection_type = connection_type.clone();
        Callback::from(move |e: Event| {
            let select = e.target_unchecked_into::<HtmlSelectElement>();
            connection_type.set(match select.value().as_str() {
                "wireless" => ConnectionType::Wireless,
                _ => ConnectionType::Wired,
            });
        })
    };

    let on_ip_mode_change = {
        let ip_mode = ip_mode.clone();
        Callback::from(move |e: Event| {
            let input = e.target_unchecked_into::<HtmlInputElement>();
            ip_mode.set(match input.value().as_str() {
                "static" => IpMode::Static,
                _ => IpMode::Dhcp,
            });
        })
    };

    let on_ssid_input = {
        let ssid = ssid.clone();
        Callback::from(move |e: InputEvent| {
            ssid.set(e.target_unchecked_into::<HtmlInputElement>().value());
        })
    };

    let on_password_input = {
        let password = password.clone();
        Callback::from(move |e: InputEvent| {
            password.set(e.target_unchecked_into::<HtmlInputElement>().value());
        })
    };

    let on_ip_address_input = {
        let ip_address = ip_address.clone();
        Callback::from(move |e: InputEvent| {
            ip_address.set(e.target_unchecked_into::<HtmlInputElement>().value());
        })
    };

    let on_gateway_input = {
        let gateway = gateway.clone();
        Callback::from(move |e: InputEvent| {
            gateway.set(e.target_unchecked_into::<HtmlInputElement>().value());
        })
    };

    let on_net_mask_input = {
        let net_mask = net_mask.clone();
        Callback::from(move |e: InputEvent| {
            net_mask.set(e.target_unchecked_into::<HtmlInputElement>().value());
        })
    };

    let on_dismiss_toast = {
        let toast = toast.clone();
        Callback::from(move |_| toast.set(None))
    };

    let navigator = use_navigator().unwrap();
    let on_configure = {
        let connection_type = connection_type.clone();
        let ip_mode = ip_mode.clone();
        let ssid = ssid.clone();
        let password = password.clone();
        let ip_address = ip_address.clone();
        let gateway = gateway.clone();
        let net_mask = net_mask.clone();
        let toast = toast.clone();
        let submitting = submitting.clone();

        Callback::from(move |_| {
            let conn_type = *connection_type;
            let ip_m = *ip_mode;
            let ssid_val = (*ssid).clone();
            let password_val = (*password).clone();
            let ip_addr_val = (*ip_address).clone();
            let gateway_val = (*gateway).clone();
            let net_mask_val = (*net_mask).clone();
            let toast = toast.clone();
            let submitting = submitting.clone();
            let navigator = navigator.clone();

            submitting.set(true);

            spawn_local(async move {
                let ip_config = match ip_m {
                    IpMode::Dhcp => IpConfig::Dhcp,
                    IpMode::Static => {
                        let ip = match Ipv4Addr::from_str(&ip_addr_val) {
                            Ok(v) => v,
                            Err(_) => {
                                toast.set(Some("Invalid IP address".to_string()));
                                submitting.set(false);
                                return;
                            }
                        };
                        let gw = match Ipv4Addr::from_str(&gateway_val) {
                            Ok(v) => v,
                            Err(_) => {
                                toast.set(Some("Invalid gateway address".to_string()));
                                submitting.set(false);
                                return;
                            }
                        };
                        let nm = match Ipv4Addr::from_str(&net_mask_val) {
                            Ok(v) => v,
                            Err(_) => {
                                toast.set(Some("Invalid network mask".to_string()));
                                submitting.set(false);
                                return;
                            }
                        };
                        IpConfig::Static(StaticIpConfig {
                            ip_address: ip,
                            net_mask: nm,
                            gateway: gw,
                            dns: vec![],
                        })
                    }
                };

                let config = match conn_type {
                    ConnectionType::Wired => NetworkConfig::Wired(WiredConfig { ip_config }),
                    ConnectionType::Wireless => NetworkConfig::Wireless(WirelessConfig {
                        ssid: ssid_val,
                        password: SecretString::from(password_val),
                        ip_config,
                    }),
                };

                let request = match Request::post("/provision/network").json(&config) {
                    Ok(r) => r,
                    Err(e) => {
                        toast.set(Some(format!("Failed to serialize request: {e}")));
                        submitting.set(false);
                        return;
                    }
                };

                if let Err(e) = request.send().await {
                    toast.set(Some(format!("Request failed: {e}")));
                    submitting.set(false);
                    return;
                }

                if let Ok(status_resp) = Request::get("/provision/status").send().await
                    && let Ok(status) = status_resp.json::<leap_api::types::ProvisionStatus>().await
                {
                    navigator.replace(&Route::from(status));
                    submitting.set(false);
                }
            });
        })
    };

    html! {
        <div class="page network-config-page">
            <h1>{ "Network Configuration" }</h1>
            if let Some(msg) = (*toast).clone() {
                <div class="toast toast-error">
                    <span>{ msg }</span>
                    <button onclick={on_dismiss_toast}>{ "✕" }</button>
                </div>
            }
            <div class="form">
                <div class="form-field">
                    <label for="connection-type">{ "Connection type" }</label>
                    <select id="connection-type" onchange={on_connection_change}>
                        <option value="wired" selected=true>{ "Wired" }</option>
                        <option value="wireless">{ "Wireless" }</option>
                    </select>
                </div>

                if *connection_type == ConnectionType::Wireless {
                    <div class="form-field">
                        <label for="ssid">{ "SSID" }</label>
                        <input id="ssid" type="text" placeholder="Network name"
                            value={(*ssid).clone()} oninput={on_ssid_input} />
                    </div>
                    <div class="form-field">
                        <label for="password">{ "Password" }</label>
                        <input id="password" type="password" placeholder="Network password"
                            value={(*password).clone()} oninput={on_password_input} />
                    </div>
                }

                <div class="form-field">
                    <label>{ "IP configuration" }</label>
                    <div class="radio-group">
                        <label class="radio-label">
                            <input type="radio" name="ip_mode" value="dhcp"
                                checked={*ip_mode == IpMode::Dhcp}
                                onchange={on_ip_mode_change.clone()} />
                            { "DHCP" }
                        </label>
                        <label class="radio-label">
                            <input type="radio" name="ip_mode" value="static"
                                checked={*ip_mode == IpMode::Static}
                                onchange={on_ip_mode_change} />
                            { "Manual" }
                        </label>
                    </div>
                </div>

                if *ip_mode == IpMode::Static {
                    <div class="form-field">
                        <label for="ip-address">{ "IPv4 address" }</label>
                        <input id="ip-address" type="text" placeholder="192.168.1.100"
                            value={(*ip_address).clone()} oninput={on_ip_address_input} />
                    </div>
                    <div class="form-field">
                        <label for="gateway">{ "Gateway" }</label>
                        <input id="gateway" type="text" placeholder="192.168.1.1"
                            value={(*gateway).clone()} oninput={on_gateway_input} />
                    </div>
                    <div class="form-field">
                        <label for="net-mask">{ "Network mask" }</label>
                        <input id="net-mask" type="text" placeholder="255.255.255.0"
                            value={(*net_mask).clone()} oninput={on_net_mask_input} />
                    </div>
                }

                <div class="form-actions">
                    <button class="btn-primary" onclick={on_configure}
                        disabled={*submitting}>
                        { "Configure" }
                    </button>
                </div>
            </div>
        </div>
    }
}
