# SSH Monitor

A terminal-based SSH monitoring tool that provides real-time system metrics visualization for remote hosts. Built with [Ratatui] for an interactive terminal user interface.

## Features

- Automatically discovers SSH hosts from your SSH config
- Monitor system metrics including:
  - CPU usage and timeline
  - Memory utilization
  - Disk usage
  - GPU metrics (if available)

## Screenshots

### Host List View
The main interface showing all discovered SSH hosts with their current system metrics:

![SSH Hosts Overview](assets/listview.png)

### Detailed Host View  
Comprehensive system monitoring with timeline charts for CPU, memory, GPU, and disk usage:

![Host Details](assets/detailview.png)


### SSH Configuration

The application reads SSH hosts from your SSH config file (typically `~/.ssh/config`). Ensure your hosts are properly configured with connection details.

Example SSH config entry:
```
Host myserver
    HostName 192.168.1.100
    User username
    Port 22
```

## Setup

### Prerequisites

- Rust (2024 edition)
- SSH access to the hosts you want to monitor
- SSH config file with host configurations

### Installation

1. Clone the repository:
```bash
git clone https://github.com/tsugumi-sys/ssh-monitor.git
cd ssh-monitor
```

2. Build the application:
```bash
cargo build --release
```

3. Run the application:
```bash
cargo run --release
```

### SSH Agent Setup

For seamless authentication, it's recommended to use SSH agent to manage your SSH keys:

#### Start SSH Agent
```bash
# Start ssh-agent (if not already running)
eval "$(ssh-agent -s)"
```

#### Add SSH Keys
```bash
# Add your private key to ssh-agent
ssh-add ~/.ssh/id_rsa

# Or add a specific key
ssh-add ~/.ssh/your_private_key

# Verify keys are loaded
ssh-add -l
```

#### SSH Config with Agent Forwarding
For enhanced security and convenience, configure your SSH hosts with agent forwarding:

```
Host myserver
    HostName 192.168.1.100
    User username
    Port 22
    ForwardAgent yes
    AddKeysToAgent yes
```

#### Persistent SSH Agent (Optional)
To automatically start ssh-agent on login, add to your shell profile (`~/.bashrc`, `~/.zshrc`, etc.):

```bash
# Auto-start ssh-agent
if [ -z "$SSH_AUTH_SOCK" ] ; then
    eval "$(ssh-agent -s)"
fi
```

### Usage

- **List View**: Browse available SSH hosts from your SSH config
- **Arrow Keys**: Navigate between hosts  
- **Enter**: View detailed metrics for selected host
- **Escape/q**: Return to previous view or quit
- **Tab**: Switch between different metric views in detail mode

## Development

For development information including architecture, testing, and contribution guidelines, see [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md).

## License

Copyright (c) Akira Noda <tidemark0105@gmail.com>

This project is licensed under the MIT license ([LICENSE] or <http://opensource.org/licenses/MIT>)

[LICENSE]: ./LICENSE
