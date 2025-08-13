use crate::ssh_config::SshHostInfo;
use ssh2::Session;
use std::io::Read;
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::time::Duration;

pub fn connect_ssh_session(info: &SshHostInfo) -> Result<Session, String> {
    let socket_addr = match info.ip.as_str() {
        "localhost" => "127.0.0.1".to_string(),
        _ => info.ip.clone(),
    };

    let socket_addr = format!("{}:{}", socket_addr, info.port)
        .parse::<SocketAddr>()
        .map_err(|e| format!("Invalid address: {}", e))?;
    let tcp = TcpStream::connect_timeout(&socket_addr, Duration::from_secs(1))
        .map_err(|e| format!("TCP error: {}", e))?;

    let mut session = Session::new().map_err(|e| format!("Session error: {}", e))?;

    session.set_tcp_stream(tcp);
    session
        .handshake()
        .map_err(|e| format!("Handshake error: {}", e))?;

    let identity_path = PathBuf::from(&info.identity_file);
    if !identity_path.exists() {
        return Err(format!(
            "Identity file not found: {}",
            identity_path.display()
        ));
    }

    let mut agent = session.agent().map_err(|e| format!("Agent error: {}", e))?;

    agent
        .connect()
        .map_err(|e| format!("Agent connect error: {}", e))?;
    agent
        .list_identities()
        .map_err(|e| format!("Agent list error: {}", e))?;

    for identity in agent.identities().unwrap_or_default() {
        if agent.userauth(&info.user, &identity).is_ok() && session.authenticated() {
            return Ok(session);
        }
    }

    Err("SSH authentication failed".into())
}

pub fn run_ssh_command(session: &Session, command: &str) -> Result<String, String> {
    let mut channel = session
        .channel_session()
        .map_err(|e| format!("Channel error: {}", e))?;
    channel
        .exec(command)
        .map_err(|e| format!("Exec error: {}", e))?;

    let mut output = String::new();
    channel
        .read_to_string(&mut output)
        .map_err(|e| format!("Read error: {}", e))?;
    channel
        .wait_close()
        .map_err(|e| format!("Wait close error: {}", e))?;

    Ok(output)
}
