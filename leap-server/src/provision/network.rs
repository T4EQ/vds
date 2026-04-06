//!

use anyhow::Context;
use secrecy::ExposeSecret;
use uuid::Uuid;

use crate::provision::network::dbus::{
    ConnectionSettings, DeviceRef, DeviceType, Ipv6Settings, WirelessMode,
};

mod dbus;

const LEAP_SETUP_UUID: Uuid = uuid::uuid!("830c6fe9-88f3-4097-b744-dce0a54db819");
const LEAP_NETWORK_UUID: Uuid = uuid::uuid!("f620adef-9c55-4be8-bf1d-f00364b5be22");

fn leap_setup_connection_settings(wifi_iface: &str) -> dbus::ConnectionSettings {
    dbus::ConnectionSettings {
        id: "leap-hotspot".to_owned(),
        uuid: LEAP_SETUP_UUID,
        interface_name: Some(wifi_iface.to_owned()),
        autoconnect: false,
        connection_type: dbus::ConnectionTypeConfig::Wireless {
            ssid: b"LEAP-setup".to_vec(),
            password: "Tech4Equality".to_owned(),
            mode: dbus::WirelessMode::AccessPoint,
        },
        ipv4_settings: Some(dbus::Ipv4Settings {
            method: dbus::Ipv4Method::Shared,
        }),
        ipv6_settings: Some(dbus::Ipv6Settings {
            method: dbus::Ipv6Method::Disabled,
        }),
    }
}

impl From<&leap_api::types::IpConfig> for dbus::Ipv4Settings {
    fn from(value: &leap_api::types::IpConfig) -> Self {
        match value {
            leap_api::types::IpConfig::Dhcp => Self {
                method: dbus::Ipv4Method::Auto,
            },
            leap_api::types::IpConfig::Static(leap_api::types::StaticIpConfig {
                ip_address,
                net_mask,
                gateway,
                // FIXME: Make use or remove
                dns,
            }) => Self {
                method: dbus::Ipv4Method::Manual {
                    ip_address: *ip_address,
                    subnet_mask: *net_mask,
                    gateway_address: *gateway,
                },
            },
        }
    }
}

impl dbus::ConnectionSettings {
    fn from_network_config(device: &DeviceRef, config: &leap_api::types::NetworkConfig) -> Self {
        let id = "leap-network".to_owned();
        match config {
            leap_api::types::NetworkConfig::Wired(leap_api::types::WiredConfig { ip_config }) => {
                Self {
                    id,
                    uuid: LEAP_NETWORK_UUID,
                    autoconnect: true,
                    interface_name: Some(device.interface.clone()),
                    connection_type: dbus::ConnectionTypeConfig::Wired,
                    ipv4_settings: Some(ip_config.into()),
                    ipv6_settings: Some(Ipv6Settings {
                        method: dbus::Ipv6Method::Disabled,
                    }),
                }
            }
            leap_api::types::NetworkConfig::Wireless(leap_api::types::WirelessConfig {
                ssid,
                password,
                ip_config,
            }) => Self {
                id,
                uuid: LEAP_NETWORK_UUID,
                autoconnect: true,
                interface_name: Some(device.interface.clone()),
                connection_type: dbus::ConnectionTypeConfig::Wireless {
                    ssid: ssid.as_bytes().to_vec(),
                    password: password.expose_secret().to_string(),
                    mode: WirelessMode::Infrastructure,
                },
                ipv4_settings: Some(ip_config.into()),
                ipv6_settings: Some(Ipv6Settings {
                    method: dbus::Ipv6Method::Disabled,
                }),
            },
        }
    }
}

fn start_provision_network_impl(
    nm: &dbus::NetworkManager,
    devs: &[dbus::DeviceRef],
) -> anyhow::Result<()> {
    if let Some(wifi_dev) = devs.iter().find(|dev| dev.dev_type == DeviceType::Wifi) {
        let connection = if let Some(connection) = nm.find_connection(&LEAP_SETUP_UUID)? {
            connection
        } else {
            let config = leap_setup_connection_settings(&wifi_dev.interface);
            nm.init_connection(&config)?
        };

        nm.activate_connection(&connection, wifi_dev)?;
    }
    Ok(())
}

pub async fn start_provision_network() -> anyhow::Result<()> {
    tokio::task::spawn_blocking(|| -> anyhow::Result<()> {
        let nm = dbus::NetworkManager::new()?;
        let devs = nm.list_devices()?;

        start_provision_network_impl(&nm, &devs)
    })
    .await??;

    Ok(())
}

pub async fn test_and_create_network_config(
    network_config: &leap_api::types::NetworkConfig,
) -> anyhow::Result<()> {
    let network_config = network_config.clone();
    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        let nm = dbus::NetworkManager::new()?;

        if let Some(old_connection) = nm.find_connection(&LEAP_NETWORK_UUID)? {
            tracing::info!("Destroying previous connection: {old_connection:?}");
            nm.delete_connection(old_connection)?;
        }

        let devs = nm.list_devices()?;
        let wireless_dev = devs
            .iter()
            .find(|d| d.dev_type == DeviceType::Wifi)
            .context("Unable to find Wireless Device!")?;
        let wired_dev = devs
            .iter()
            .find(|d| d.dev_type == DeviceType::Ethernet)
            .context("Unable to find Wired Device!")?;

        let device = if network_config.is_wired() {
            wired_dev
        } else {
            wireless_dev
        };

        let config = ConnectionSettings::from_network_config(device, &network_config);
        let connection = nm.init_connection(&config)?;

        // Check conection by activating it
        let active_connection = nm.activate_connection(&connection, device)?;

        const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
        let start = std::time::Instant::now();
        'outer: {
            while std::time::Instant::now() - start < TIMEOUT {
                let status = nm.connection_status(&active_connection)?;
                if status == dbus::ConnectionState::Activated {
                    break 'outer;
                }

                std::thread::sleep(std::time::Duration::from_millis(10));
            }

            // Restart provision network
            start_provision_network_impl(&nm, &devs)?;
            anyhow::bail!("Connection did not activate successfully");
        }

        nm.save_connection(&connection)?;

        // Re-enable the setup connection to complete the rest of the setup
        start_provision_network_impl(&nm, &devs)?;

        Ok(())
    })
    .await??;

    Ok(())
}

pub async fn temporarily_enable_network_config<T, U>(action: T) -> anyhow::Result<()>
where
    T: FnOnce() -> U,
    U: Future<Output = anyhow::Result<()>>,
{
    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        let nm = dbus::NetworkManager::new()?;

        tracing::info!("Finding leap network configuration.");
        let Some(connection) = nm.find_connection(&LEAP_NETWORK_UUID)? else {
            tracing::error!("No network connection has been configured!");
            anyhow::bail!("No network connection has been configured!");
        };

        tracing::info!("Finding wireless and wired devices");
        let devs = nm.list_devices()?;
        let wireless_dev = devs
            .iter()
            .find(|d| d.dev_type == DeviceType::Wifi)
            .context("Unable to find Wireless Device!")?;
        let wired_dev = devs
            .iter()
            .find(|d| d.dev_type == DeviceType::Ethernet)
            .context("Unable to find Wired Device!")?;

        let connection_type = nm.query_connection_type(&connection)?;
        tracing::info!("Got connection type: {connection_type:?}");
        let device = match connection_type {
            dbus::ConnectionType::Wired => wired_dev,
            dbus::ConnectionType::Wireless => wireless_dev,
        };

        tracing::info!("Activating leap network connection");
        let active_connection = nm.activate_connection(&connection, device)?;
        tracing::info!("Waiting for up to 30 seconds for the network to connect");
        const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
        let start = std::time::Instant::now();
        'outer: {
            while std::time::Instant::now() - start < TIMEOUT {
                let status = nm.connection_status(&active_connection)?;
                if status == dbus::ConnectionState::Activated {
                    break 'outer;
                }

                std::thread::sleep(std::time::Duration::from_millis(10));
            }

            // Restart provision network
            start_provision_network_impl(&nm, &devs)?;
            tracing::error!("Leap network connection failed to connect");
            anyhow::bail!("Connection did not activate successfully");
        }

        Ok(())
    })
    .await??;

    tracing::info!("Leap network is enabled.");
    let action_result = action().await;
    tracing::info!("Completed action requiring network connectivity.");

    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        let nm = dbus::NetworkManager::new()?;
        let devs = nm.list_devices()?;
        tracing::info!("Restarting leap provision network config");
        start_provision_network_impl(&nm, &devs)?;
        tracing::info!("Restarted leap provision network config");
        Ok(())
    })
    .await??;

    action_result
}
