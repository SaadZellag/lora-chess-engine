# SPRT Test Manager

A web-based GUI application for managing and running SPRT (Sequential Probability Ratio Test) tests using fastchess, with support for engine compilation from git branches/commits and Discord webhook notifications.

## Features

- 🎯 **Create SPRT Tests**: Specify two engines by branch name or commit hash
- 🔧 **Compiler Environment Variables**: Set custom env vars (e.g., NNUE paths) for compilation
- ⚙️ **Fastchess Parameters**: Configure all fastchess test parameters
- 📊 **Monitor Tests**: Real-time view of running, paused, finished, or failed tests
- ⏸️ **Control Tests**: Pause, resume, or discard any test
- 📝 **Live Logs**: View detailed logs for each test
- 💾 **Binary Caching**: Compiled engines are cached to avoid recompilation
- 📢 **Discord Notifications**: Get notified of test events via Discord webhooks
- 💾 **Persistent Settings**: Default parameters persist across reboots
- 🎨 **Modern UI**: Beautiful Vue 3 interface

## Architecture

### Backend
- **Rust** with Actix-web framework
- **SQLite** for persistent storage
- Manages engine compilation, caching, and fastchess execution
- Discord webhook integration

### Frontend
- **Vue 3** with modern CSS styling
- Responsive design for desktop and mobile
- Real-time test status updates

## Setup

### Prerequisites
- Rust 1.70+ and Cargo
- Git
- A working Lora chess engine repository
- fastchess binary (download or build from source)

### Installation

1. Build the project:
```bash
cd sprt-test-manager
cargo build --release
```

2. Create a `.env` file (or use defaults):
```bash
cp .env.example .env
```

3. Edit `.env` with your paths:
```env
DB_PATH=sprt_tests.db
HOST=0.0.0.0
PORT=8000
```

4. Run the application:
```bash
cargo run --release
```

5. Open your browser and navigate to:
```
http://localhost:8000
```

## Configuration

### Initial Setup

On first load, you need to configure the settings in the UI:

1. Go to **Settings** tab
2. Set the paths:
   - **Compiled Engines Path**: Directory where compiled engines will be cached
   - **Lora Repository Path**: Path to your Lora engine repository
   - **Fastchess Path**: Path to the fastchess binary
3. Set default fastchess parameters (JSON format)
4. Set default compiler environment variables (JSON format)
5. Optionally set a default Discord webhook URL

### Fastchess Parameters

All fastchess parameters are configured via JSON. Common examples:

```json
{
  "rounds": 100,
  "repeat": true,
  "concurrency": 4,
  "sprt": "elo0=0 elo1=2 alpha=0.05 beta=0.05",
  "tc": "60+0.6",
  "openings": "file=book.epd format=epd order=random",
  "draw": "movenumber=34 movecount=8 score=20",
  "resign": "movecount=3 score=600 twosided=true"
}
```

See the fastchess documentation in the prompts for all available parameters.

### Discord Notifications

Set a Discord webhook URL to receive notifications for:
- Test started
- Test stopped/paused
- Test resumed
- Test completed
- Test failed

To create a Discord webhook:
1. Go to your Discord server settings
2. Select "Webhooks"
3. Create a new webhook
4. Copy the webhook URL
5. Paste it in the app settings or per-test configuration

## Using the Application

### Creating a Test

1. Click the **Create Test** tab
2. Enter engine names and git references (branch or commit hash)
3. Optionally set custom compiler environment variables (JSON)
4. Configure fastchess parameters
5. Optionally set a Discord webhook for notifications
6. Click **Start Test**

The application will:
1. Git checkout to the specified refs
2. Compile both engines with the specified env vars
3. Cache the compiled binaries
4. Run fastchess with the configured parameters
5. Display real-time progress

### Monitoring Tests

1. Click the **Running Tests** tab
2. View all tests with their current status
3. See progress (games played vs total)
4. View detailed logs by clicking **Logs**
5. Pause or resume tests as needed
6. Discard tests if needed

### Managing Settings

Settings are automatically persisted to the SQLite database. Changes take effect immediately and survive application restarts.

## Project Structure

```
sprt-test-manager/
├── src/
│   ├── main.rs              # Server setup and routing
│   ├── db.rs                # SQLite database operations
│   ├── models.rs            # Data structures
│   ├── config.rs            # Configuration management
│   ├── errors.rs            # Error types
│   ├── engine.rs            # Engine compilation and caching
│   ├── fastchess.rs         # Fastchess execution
│   ├── discord.rs           # Discord webhook notifications
│   └── handlers/
│       ├── mod.rs
│       ├── tests.rs         # Test API endpoints
│       ├── settings.rs      # Settings API endpoints
│       └── static_files.rs  # Static file serving
├── static/
│   ├── index.html           # Main UI
│   ├── app.js               # Vue 3 application
│   └── style.css            # Styling
└── Cargo.toml
```

## API Endpoints

### Tests
- `GET /api/tests` - List all tests
- `POST /api/tests` - Create a new test
- `GET /api/tests/{id}` - Get test details
- `DELETE /api/tests/{id}` - Discard a test
- `POST /api/tests/{id}/pause` - Pause a test
- `POST /api/tests/{id}/resume` - Resume a test
- `GET /api/tests/{id}/logs` - Get test logs

### Settings
- `GET /api/settings` - Get current settings
- `PUT /api/settings` - Update settings

## Building for Production

```bash
cargo build --release
```

The compiled binary will be in `target/release/sprt-test-manager`.

## Troubleshooting

### Compilation fails
- Check that the lora repo path is correct
- Verify git can access the repository
- Check environment variables (NNUE_PATH, etc.)
- Review test logs for detailed error messages

### Fastchess not found
- Set the correct path to fastchess in settings
- Ensure fastchess is executable
- Try using an absolute path

### Discord notifications not working
- Verify the webhook URL is correct
- Check that the webhook channel permissions allow the bot to post
- Look at application logs for error messages

## Development

### Building locally
```bash
cargo build
```

### Running with debug logging
```bash
RUST_LOG=debug cargo run
```

### Running tests
```bash
cargo test
```

## License

MIT

## Future Enhancements

- Multiple engine support (beyond just Lora)
- Test result analysis and visualization
- Engine rating/ELO tracking
- Batch test scheduling
- Test result comparison
- PGN export and analysis
- WebSocket real-time updates
- Authentication and multi-user support
