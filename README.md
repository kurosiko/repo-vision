# repo-vision

A terminal UI tool for bulk managing GitHub repository visibility settings. Quickly browse, filter, and toggle repositories between public and private in an interactive TUI powered by Ratatui.

## What this is

`repo-vision` is a Rust-based CLI application that helps you manage the visibility (public/private) of your GitHub repositories at scale. Instead of toggling visibility one repository at a time through the GitHub web interface, you can browse all your repos in an interactive terminal UI, select the ones you want to change, and apply visibility changes in bulk.

### Stack

- **Language:** Rust (100%)
- **Framework / runtime:** Tokio (async runtime) + Ratatui (terminal UI)
- **Notable libraries:**
  - **ratatui** — Terminal UI rendering and layout management
  - **crossterm** — Cross-platform terminal input/output handling
  - **github-rust** — GitHub API integration
  - **serde/serde_json** — JSON serialization for GitHub API responses
  - **color-eyre** — Error reporting and formatting

## How it's organized

```
src/
  main.rs       Entry point, GitHub API integration, repo data structures
  ui.rs         Terminal UI state management, rendering, and key handlers
```

**How it fits together:**

1. **Startup** (`main.rs`): The application initializes the error handler and fetches your repositories from GitHub using the `gh repo list` command.

2. **UI State Loop** (`ui.rs`): The app enters an interactive loop where it renders a table of repositories and handles keyboard input. Users can filter repos (by public/private status or star count), navigate with vim-like keybindings (j/k or arrow keys), toggle visibility with the spacebar, and preview pending changes.

3. **Change Application** (`main.rs`): When confirmed, visibility changes are applied via the `gh repo edit` command, calling GitHub's CLI under the hood.

The TUI maintains a separate `changed_indices` set to track which repos have been modified locally. When you press `y` in the confirmation popup, only the changed repos are sent to GitHub.

## How to run it

### Prerequisites

- Rust 1.70+ (or use `devbox`)
- `gh` CLI installed and authenticated (`gh auth login`)

### Quick start

```bash
# Clone the repository
git clone https://github.com/kurosiko/repo-vision
cd repo-vision

# Run with Cargo
cargo run --release

# Or run tests
cargo test
```

### Using devbox (recommended)

```bash
# Initialize devbox environment
devbox run bash

# Then inside devbox:
cargo run --release
```

### Key bindings

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `g` / `Home` | Jump to first repo |
| `G` / `End` | Jump to last repo |
| `Tab` | Cycle through filters (Public → Private → Starred → All) |
| `Space` | Toggle visibility of selected repo (local only) |
| `p` | Preview pending changes |
| `y` | Apply pending changes to GitHub (in preview popup) |
| `q` / `Esc` | Quit |

## Try asking

- How does the TUI filter and display different repository visibility types?
- Where is the GitHub API communication happening, and what `gh` commands are used?
- How are the pending changes tracked, and what prevents accidental commits?
