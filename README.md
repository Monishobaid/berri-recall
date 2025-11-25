# berri-recall

Your terminal doesn't remember what you typed last week. Mine does.

I got tired of scrolling through `history | grep` looking for that docker command I ran three days ago. Or trying to remember the exact flags for that `ffmpeg` thing. So I built this.

**berri-recall** watches everything you type and actually makes it useful. Pattern detection, fuzzy search, per-project memory. Written in Rust because honestly, why would you write a CLI tool in anything else?

---

## What it does

- Remembers every command you run. Automatically. Per project.
- Fuzzy search that actually works (unlike ctrl+r which is... fine I guess)
- Tracks whether commands failed or not (so you stop repeating broken commands)
- Detects patterns in how you work (like when you always run `npm test` after `git add`)
- Everything stays local. No cloud, no telemetry, no BS.
- Fast enough that you'll forget it's running (<10ms overhead)

Works with bash, zsh, fish, and PowerShell. Yes, even PowerShell.

---

## Install it

Pick whatever works for you:

### One-line install (Mac & Linux)

```bash
curl -fsSL https://raw.githubusercontent.com/monishobaid/berri-recall/main/install.sh | bash
```

Then run `berri-recall setup` and restart your shell.

### Homebrew (Mac)

```bash
brew tap monishobaid/berri-recall
brew install berri-recall
berri-recall setup
```

### From GitHub releases

Grab the latest binary for your platform:
- [macOS (Apple Silicon)](https://github.com/monishobaid/berri-recall/releases/latest/download/berri-recall-macos-arm64.tar.gz)
- [macOS (Intel)](https://github.com/monishobaid/berri-recall/releases/latest/download/berri-recall-macos-amd64.tar.gz)
- [Linux](https://github.com/monishobaid/berri-recall/releases/latest/download/berri-recall-linux-amd64.tar.gz)
- [Windows](https://github.com/monishobaid/berri-recall/releases/latest/download/berri-recall-windows-amd64.zip)

Extract it and move to `/usr/local/bin` (or wherever you keep binaries).

### From source

```bash
git clone https://github.com/monishobaid/berri-recall.git
cd berri-recall/src-tauri
cargo install --path .
berri-recall setup
```

After installing, run `berri-recall setup` and restart your shell. That's it.

---

## Actually using it

Once you've run setup, it just works. Type commands like normal:

```bash
$ npm test
$ git commit -m "finally fixed that weird bug"
$ cargo build --release
```

All recorded. Didn't even notice, right?

### See what you've been running

```bash
$ berri-recall recent

Recent commands:
============================================================
  1. ✓ npm test (used 5 times)
  2. ✗ cargo build (used 2 times)  # this one failed btw
  3. ✓ git status (used 10 times)
============================================================
```

The little ✓/✗ tells you which commands actually worked.

### Search for something

```bash
$ berri-recall search docker

Found 3 command(s) matching 'docker':
============================================================
  1. docker-compose up (used 5 times)
  2. docker ps (used 3 times)
  3. docker logs app (used 2 times)
============================================================
```

Fuzzy search works here. Type `dcr` and it'll find `docker-compose run`. I don't know how I lived without this.

### Check if it's actually working

```bash
$ berri-recall status

berri-recall Status
============================================================

Shell Hooks:
  bash:        ✗ Not installed
  zsh:         ✓ Installed  # you're good
  fish:        ✗ Not installed
  powershell:  ✗ Not installed

Database Statistics:
  Commands:    127  # yeah you type a lot
  Patterns:    0
  Suggestions: 0

Current Shell:
  zsh
============================================================
```

---

## All the commands

```bash
# Setup (do this once)
berri-recall setup              # figures out your shell automatically
berri-recall setup --all        # install for every shell you have

# Looking stuff up
berri-recall recent             # last 10 commands
berri-recall recent 20          # last 20 commands
berri-recall search npm         # find anything with "npm" in it

# If you're old school and don't want auto-recording
berri-recall record "npm test"  # manually save a command

# Maintenance
berri-recall status             # see what's happening
berri-recall uninstall          # remove all the hooks
berri-recall version            # current version
berri-recall help               # you know what this does
```

---

## How this actually works

Your shell has hooks that fire after every command. I tap into those hooks and record:
- The command you typed
- Whether it worked or failed (exit code)
- Which project you're in (looks for .git folders)
- Timestamp

Everything gets shoved into a SQLite database at `~/.berri-recall/commands.db`. Runs in the background so it doesn't slow you down.

**Bash** uses `PROMPT_COMMAND`. **Zsh** uses `preexec` and `precmd` (which are honestly better). **Fish** has `fish_postexec`. **PowerShell** does its own thing with `PSReadLine`.

None of this blocks your terminal. You won't even notice it's running.

---

## Privacy stuff

All your data lives in `~/.berri-recall/` on your machine. That's it.

- No cloud sync
- No telemetry
- No network calls
- Open source so you can read every line

It also filters out sensitive stuff automatically:

```bash
$ mysql -u root --password=secret123
# NOT recorded - detected password flag

$ export API_KEY=abc123
# NOT recorded - looks like a secret

$ npm install
# Recorded - this is fine
```

I'm paranoid about this stuff too.

---

## Building from source

Need Rust 1.70 or newer. Get it from [rustup.rs](https://rustup.rs/).

```bash
git clone https://github.com/monishobaid/berri-recall.git
cd berri-recall/src-tauri

cargo build              # dev build
cargo build --release    # optimized build
cargo test               # run tests
cargo clippy             # check for issues
```

The release binary ends up in `target/release/berri-recall`.

---

## Project structure

```
berri-recall/
├── src-tauri/          # All the Rust code
│   ├── src/
│   │   ├── core/       # Recording and retrieval logic
│   │   ├── db/         # SQLite stuff
│   │   ├── shell/      # Shell detection and hook installation
│   │   ├── intelligence/ # Pattern detection, suggestions
│   │   └── main.rs     # CLI entry point
│   └── Cargo.toml
├── hooks/              # Shell integration scripts
│   ├── bash.sh
│   ├── zsh.sh
│   ├── fish.fish
│   └── powershell.ps1
└── database/
    └── schema.sql      # Database schema
```


## Performance

I benchmarked this because I was curious:

- Recording a command: <10ms (you literally can't feel this)
- Searching: <100ms
- Binary size: ~8MB
- Memory: <10MB RAM

Rust is stupid fast for stuff like this.

---

## Troubleshooting

**Hooks not recording?**

```bash
berri-recall status          # check what's installed
berri-recall uninstall       # nuke it
berri-recall setup           # try again
source ~/.zshrc              # reload your shell
```

**Nothing showing up?**

```bash
berri-recall recent          # see if anything's there
berri-recall record "test"   # manually record something
berri-recall recent          # check again
```

**Want to start over?**

```bash
rm -rf ~/.berri-recall       # delete everything
berri-recall setup           # reinstall
```

---

## Contributing

Pull requests are open. If you want to add something:

1. Make sure it builds (`cargo build`)
2. Run the tests (`cargo test`)
3. Don't make the code ugly
4. Send a PR

Things I'd love help with:
- Better pattern detection algorithms
- More shell support (nushell? xonsh?)
- Performance improvements (it's already fast but faster is better)
- Bug fixes

---

## Tech stack

- **Rust** - obvious choice for a CLI tool
- **SQLx** - type-safe SQL queries (no ORMs, those are slow)
- **Tokio** - async runtime
- **SQLite** - embedded database (no setup required)

---

## License

MIT. Do whatever you want with it.

---

## Support

Something broken? [Open an issue](https://github.com/monishobaid/berri-recall/issues).

Want to chat about it? [Start a discussion](https://github.com/monishobaid/berri-recall/discussions).

---

Built by [Monish Obaid](https://github.com/monishobaid) because terminal history is basically useless.
