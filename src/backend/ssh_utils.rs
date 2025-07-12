use crate::app::states::SshHostInfo;
use ssh2::Session;
use std::io::Read;
use std::net::TcpStream;
use std::path::PathBuf;

/// Tries to establish an authenticated SSH session.
/// Returns `Session` on success or an error string on failure.
pub fn connect_ssh_session(info: &SshHostInfo) -> Result<Session, String> {
    let addr = format!("{}:{}", info.ip, info.port);
    let tcp = TcpStream::connect(&addr).map_err(|e| format!("TCP error: {}", e))?;

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

pub fn run_command(session: &Session, command: &str) -> Result<String, String> {
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
