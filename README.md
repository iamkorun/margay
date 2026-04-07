<p align="center">
  <h1 align="center">margay 🐱</h1>
  <p align="center">Audit (and fix) file permissions across your project tree</p>
</p>

<p align="center">
  <a href="https://github.com/iamkorun/margay/actions/workflows/ci.yml"><img src="https://github.com/iamkorun/margay/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://crates.io/crates/margay"><img src="https://img.shields.io/crates/v/margay.svg" alt="crates.io"></a>
  <a href="https://github.com/iamkorun/margay/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
  <a href="https://github.com/iamkorun/margay/stargazers"><img src="https://img.shields.io/github/stars/iamkorun/margay?style=social" alt="Stars"></a>
  <a href="https://buymeacoffee.com/iamkorun"><img src="https://img.shields.io/badge/Buy%20Me%20a%20Coffee-ffdd00?logo=buy-me-a-coffee&logoColor=black" alt="Buy Me a Coffee"></a>
</p>

---

<!-- TODO: Add demo GIF at docs/demo.gif -->

## The Problem

Your `deploy.sh` ships without the execute bit and CI dies on `permission denied`. Your `.env` is sitting at `0644` so every process on the box can read your Stripe key. Some intern committed `notes.md` as `0755`. You only find out when something breaks — or worse, when something leaks.

`chmod` is a footgun, and nobody audits permissions until it's already a problem.

## The Solution

**margay** walks your project (respecting `.gitignore`) and spots three classes of permission bugs:

- shell scripts missing the user-execute bit
- sensitive files (`.env`, `*.key`, `*.pem`, `id_rsa`, ...) with permissions looser than `0600`
- source files (`*.rs`, `*.py`, `*.md`, ...) accidentally marked executable

Run `margay` to audit. Run `margay --fix` to patch them all.

Named after the [margay](https://en.wikipedia.org/wiki/Margay) — a small spotted wild cat that watches over the canopy and never misses a thing.

## Demo

```text
$ margay
Sensitive files too open
  !! ./.env 0644 → 0600

Shell scripts missing +x
  +x ./deploy.sh 0644 → 0744

Source files marked executable
  -x ./main.rs 0755 → 0644

✗ 3 issues found across 3 files
```

The first column is a category marker (`!!` sensitive, `+x` should be exec, `-x` shouldn't be), then the path, the current mode, and the mode `--fix` would set.

## Quick Start

```sh
# install (from source until the crate is published)
cargo install --git https://github.com/iamkorun/margay

# audit your project
cd your-project/
margay

# auto-correct everything
margay --fix
```

## Installation

### From source (works today)

```sh
cargo install --git https://github.com/iamkorun/margay
```

or clone and build:

```sh
git clone https://github.com/iamkorun/margay.git
cd margay
cargo install --path .
```

### From crates.io (after publish)

```sh
cargo install margay
```

### Binary releases

Pre-built binaries for Linux and macOS will be published to the [Releases](https://github.com/iamkorun/margay/releases) page once the first tagged release lands.

## Usage

### Audit the current directory

```sh
margay
```

### Audit a specific path

```sh
margay ./services/api
```

### Auto-fix every issue

```sh
margay --fix
# margay: fixed 5 issue(s)
```

### Machine-readable JSON for CI

```sh
margay --json
```

```json
{
  "total": 3,
  "issues": [
    {
      "path": "./.env",
      "category": "sensitive-too-open",
      "mode": "0644",
      "fix_mode": "0600",
      "message": "sensitive file is group/world accessible (mode 644)"
    },
    {
      "path": "./deploy.sh",
      "category": "shell-needs-exec",
      "mode": "0644",
      "fix_mode": "0744",
      "message": "shell script is not user-executable (mode 644)"
    },
    {
      "path": "./main.rs",
      "category": "source-executable",
      "mode": "0755",
      "fix_mode": "0644",
      "message": "source file is marked executable (mode 755)"
    }
  ]
}
```

`category` is one of `sensitive-too-open`, `shell-needs-exec`, or `source-executable`. `mode` and `fix_mode` are 4-digit octal strings.

### Quiet mode

`--quiet` collapses the per-category listing to a single summary line — useful when you only want the headline number in CI logs.

```sh
$ margay --quiet
✗ 3 issues found across 3 files
```

The exit code is the same in every mode:

| Code | Meaning |
|------|---------|
| `0`  | No issues (or `--fix` succeeded) |
| `1`  | One or more issues found |
| `2`  | Runtime error (path missing, etc.) |

## Flags

| Flag | Short | Description |
|------|-------|-------------|
| `--fix` |  | Auto-correct all detected issues |
| `--json` |  | Emit machine-readable JSON (mutually exclusive with `--fix`) |
| `--quiet` | `-q` | Suppress non-essential output |
| `--verbose` | `-v` | Print extra detail while scanning |
| `--help` | `-h` | Print help |
| `--version` | `-V` | Print version |

`--quiet` and `--verbose` are mutually exclusive.

## Features

- **Zero config** — point it at a directory and go
- **Respects `.gitignore`** — no noise from `target/`, `node_modules/`, `dist/`
- **Three rule classes** — exec bits, sensitive files, source-marked-exec
- **Auto-fix** — `--fix` patches every issue in one pass
- **CI-friendly** — JSON output + non-zero exit code on findings
- **Fast** — pure Rust, walks 10k files in well under a second
- **Safe by default** — never touches a file unless you ask

## CI Integration

Drop margay into GitHub Actions to catch permission regressions on every PR:

```yaml
name: Audit Permissions
on: [push, pull_request]

jobs:
  perms:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Install margay
        run: cargo install --git https://github.com/iamkorun/margay
      - name: Audit file permissions
        run: margay
```

The job fails on any finding because `margay` exits non-zero. Add `--json` if you want to pipe the result into another step.

## Contributing

Contributions welcome! Open an issue first to discuss bigger changes.

```sh
git clone https://github.com/iamkorun/margay.git
cd margay
cargo test
```

## License

[MIT](LICENSE)

---

## Star History

<a href="https://star-history.com/#iamkorun/margay&Date">
  <img src="https://api.star-history.com/svg?repos=iamkorun/margay&type=Date" alt="Star History Chart" width="600">
</a>

---

<p align="center">
  <a href="https://buymeacoffee.com/iamkorun"><img src="https://cdn.buymeacoffee.com/buttons/v2/default-yellow.png" alt="Buy Me a Coffee" width="200"></a>
</p>
