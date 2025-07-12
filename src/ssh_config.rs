use eyre::Result;
use md5;
use ssh2_config::{Host, ParseRule, SshConfig};
use std::collections::HashMap;
use std::sync::Arc;
use std::{fs::File, io::BufReader};
use tokio::sync::Mutex;

pub const PLACEHOLDER_IP: &str = "-";
pub const PLACEHOLDER_USER: &str = "-";
pub const PLACEHOLDER_PORT: u16 = 22;
pub const PLACEHOLDER_IDENTITY_FILE: &str = "-";

#[derive(Debug, Clone)]
pub struct SshHostInfo {
    pub id: String,
    pub name: String,
    pub ip: String,
    pub port: u16,
    pub user: String,
    pub identity_file: String,
}

pub type SharedSshHosts = Arc<Mutex<HashMap<String, SshHostInfo>>>;

impl SshHostInfo {}

pub fn load_ssh_configs() -> Result<HashMap<String, SshHostInfo>> {
    let path = dirs::home_dir()
        .ok_or_else(|| eyre::eyre!("Could not resolve home dir"))?
        .join(".ssh/config");

    let mut reader = BufReader::new(File::open(path)?);
    let config = SshConfig::default().parse(&mut reader, ParseRule::STRICT)?;

    let hosts = config
        .get_hosts()
        .iter()
        .filter_map(|host: &Host| {
            let name = host.pattern.first()?.to_string();
            if name == "*" {
                return None;
            }

            let ip = host
                .params
                .host_name
                .clone()
                .unwrap_or_else(|| PLACEHOLDER_IP.into());
            let user = host
                .params
                .user
                .clone()
                .unwrap_or_else(|| PLACEHOLDER_USER.into());
            let port = host.params.port.unwrap_or(PLACEHOLDER_PORT);

            let identity_file = host
                .params
                .identity_file
                .clone()
                .and_then(|list| list.first().cloned())
                .map(|pathbuf| pathbuf.to_string_lossy().into_owned())
                .unwrap_or_else(|| PLACEHOLDER_IDENTITY_FILE.into());

            let hash_input = format!("{}:{}:{}", name, ip, port);
            let id = format!("{:x}", md5::compute(hash_input));

            Some((
                id.clone(),
                SshHostInfo {
                    id,
                    name,
                    ip,
                    port,
                    user,
                    identity_file,
                },
            ))
        })
        .collect();

    Ok(hosts)
}
