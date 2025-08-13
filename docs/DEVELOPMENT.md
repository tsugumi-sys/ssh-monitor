# SSH Monitor - Development Guide

## Architecture

The application is structured with:

- **Backend**: SSH connection handling, database operations, and metric collection jobs
- **TUI**: Terminal user interface with multiple views and state management
- **Database**: SQLite for storing historical metrics data
- **Jobs System**: Async job executor for background metric collection

## Adding New Metrics

To add new database states and metrics, follow the patterns documented in `CLAUDE.md`. The process involves:

1. Adding database query methods in `src/backend/db/{module}/queries.rs`
2. Creating timeline states in `src/tui/host_details/states.rs` 
3. Updating job kinds and registering in the main application

## Testing

Run tests with debugging enabled:

```bash
RUST_LOG=debug cargo test -- --nocapture
```

## Code Quality

The project uses:
- `cargo fmt` for code formatting
- `cargo clippy` for linting
- Typos checking with `_typos.toml` configuration

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes following the existing patterns
4. Run tests and ensure code quality
5. Submit a pull request
