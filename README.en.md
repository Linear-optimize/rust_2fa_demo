# Rust 2FA · Terminal Authenticator

[简体中文](README.md) | English

A TOTP (Time-Based One-Time Password) generator written in Rust, with a Ratatui terminal interface. Enter a Base32 secret to view the current six-digit code, its remaining validity, and a countdown progress bar.

> This is a single-account, local learning project—not an encrypted secret vault. Verify its behavior with test secrets before considering it for real accounts.

## Features

- **Hidden secret input**: Enter your secret interactively after startup, without passing it through command-line arguments.
- **Input normalization**: Ignore spaces, newlines, and other whitespace, convert lowercase letters to uppercase, and remove `=` padding characters.
- **Six-digit codes**: Preserve leading zeros and display codes in groups, such as `123 456`.
- **Automatic refresh**: Generate a new code every 30 seconds; check events and redraw the interface approximately every 100 milliseconds.
- **Validity progress bar**: Display the remaining lifetime of the current code.
- **Dynamic status colors**: Green for valid codes, yellow when time is running low, and red just before refresh.
- **Dark terminal interface**: A centered, rounded card with a layout that adjusts to terminal dimensions.
- **Keyboard exit**: Press `q`, `Q`, or `Esc` to quit and restore the terminal state.

## Installation and Usage

Install Rust and Cargo, then run the application in an interactive terminal. Windows Terminal or another modern terminal supporting Unicode and true color is recommended. A window of roughly 64 columns by 21 rows can display the entire card.

From the project root, run:

```bash
cargo run
```

When prompted, type or paste the **Base32 TOTP secret** supplied by your service, then press Enter. The current application's prompt is in Chinese:

```text
请输入 Base32 密钥:
```

This means “Enter Base32 secret.” Your input will not be echoed. Use the secret provided when enabling two-factor authentication—**not your account password or a current six-digit code**.

Secrets may contain spaces or lowercase letters. For example, these inputs normalize to the same value:

```text
JBSWY3DPEHPK3PXP
jbswy3dpehpk3pxp
jbsw y3dp ehpk 3pxp
```

> This secret is for demonstration only. Public example secrets must never protect real accounts.

### Build and Run the Executable

```bash
cargo build --release
```

Windows PowerShell:

```powershell
.\target\release\rust_2fa.exe
```

Linux / macOS:

```bash
./target/release/rust_2fa
```

The executable name depends on the package or binary target name in `Cargo.toml`. These examples assume `rust_2fa`.

## Interface

| Area | Content |
| --- | --- |
| Header | Rust TOTP branding and two-factor authentication description |
| Status | Code valid, use soon, or refreshing soon |
| Current Code | The current six-digit code, shown as two groups of three digits |
| Validity | Remaining seconds and a countdown progress bar |
| Footer | Refresh information, a security reminder, and exit shortcuts |

| Time Remaining | Color | Status |
| --- | --- | --- |
| 11–30 seconds | Green | Code valid |
| 6–10 seconds | Yellow | Use soon |
| 1–5 seconds | Red | Refreshing soon |

The progress bar represents **the proportion of time left in the current window**, not application execution progress. Some status messages and hints in the current UI are in Chinese; this English README does not change the application's language.

## Project Structure

| File | Responsibility |
| --- | --- |
| `Cargo.toml` | Package metadata and dependency configuration |
| `Cargo.lock` | Locked dependency versions for reproducible builds |
| `src/main.rs` | Module declarations, hidden secret input, error handling, and UI startup |
| `src/totp.rs` | Base32 decoding, TOTP generation, and remaining-time calculation |
| `src/ui.rs` | Theme, card layout, progress bar, status colors, and keyboard events |

The documentation is available in [Simplified Chinese](README.md) and [English](README.en.md). Keep both files in the same directory for the language links to work.

## Implementation

The current implementation uses these fixed parameters:

| Parameter | Value |
| --- | --- |
| Algorithm | HMAC-SHA1 |
| Time step | 30 seconds |
| Code length | 6 digits |
| Time origin | Unix Epoch |
| Secret encoding | Base32 |

The application divides the Unix timestamp by 30 to obtain a counter, encodes it as eight big-endian bytes, and computes HMAC-SHA1 using the decoded secret. Dynamic truncation extracts a nonnegative integer, which is reduced modulo `1_000_000` to produce a six-digit code.

Remaining time in the current window is calculated as:

```rust
30 - timestamp % 30
```

### Dependencies

| Dependency | Purpose |
| --- | --- |
| `base32` | Decode Base32 secrets |
| `sha1` | SHA-1 hashing |
| `hmac` | Keyed message authentication |
| `rpassword` | Hidden secret input |
| `ratatui` | Terminal widgets and rendering |
| `crossterm` | Cross-platform terminal keyboard events |

The example UI uses Ratatui 0.30 APIs. For a new project without these dependencies, run:

```bash
cargo add base32 sha1 hmac rpassword ratatui@0.30 crossterm
```

For an existing project, prefer retaining its verified `Cargo.toml` and `Cargo.lock` rather than upgrading dependencies unnecessarily.

## Development

```bash
cargo fmt
cargo check
cargo clippy
```

Edit `src/ui.rs` to change colors, card dimensions, and status messages. Edit `src/totp.rs` to change algorithm parameters. If you change the time step, also update the UI's progress-bar ratio and related text.

## Security Notes

- **The secret is more sensitive than a short-lived code**: Anyone holding it can continue generating codes.
- Never put real secrets in source code, README files, configuration examples, Git commits, issues, or logs.
- Do not pass secrets with `cargo run -- <secret>`; command history and process arguments may expose them.
- Hidden input only suppresses character echo. It does not encrypt the secret or protect against keyloggers, malware, or memory inspection.
- The current code does not intentionally save secrets to files, but keeps them in memory and does not implement secure memory zeroization.
- When pasting secrets, consider clipboard history, cross-device clipboard synchronization, and clipboard managers.
- Codes appear on screen. Avoid recording, taking screenshots of, or sharing terminal output containing them.
- This tool generates codes only. It does not validate codes, log into accounts, or recover secrets.
- Keep the recovery codes supplied by your service. Do not rely on this tool as your only account recovery method.

## Current Limitations

- One secret per application session.
- The secret must be entered again on every launch.
- No QR-code scanning or `otpauth://` URI import.
- No secret persistence, encrypted storage, or system credential-manager integration.
- No SHA-256, SHA-512, eight-digit codes, or configurable time steps.
- The layout adjusts, but very small terminal windows may clip text or widgets.
- Codes depend on the local system clock; the tool does not automatically compensate for differences from the server's time.

## FAQ

### Why does nothing appear when I type the secret?

This is expected. `rpassword` hides input by default. Type or paste your secret, then press Enter.

### Why does my code differ from the website or another authenticator?

Confirm that the secret belongs to the same account, check that your system clock is accurate, and verify that the service uses HMAC-SHA1, six digits, and a 30-second time step. Matching digit counts alone do not imply matching parameters.

### Why is my Base32 secret rejected?

Standard Base32 secrets typically use letters `A–Z` and digits `2–7`. Make sure you have not entered a password, recovery code, QR-code payload, or complete `otpauth://` URI. The application handles whitespace, lowercase letters, and `=`, but does not parse URIs.

### Why is `new_from_slice` not found?

For the `hmac` API used by the current example, import the trait providing this method in `src/totp.rs`:

```rust
use hmac::{Hmac, KeyInit, Mac};
```

Trait APIs may differ between versions. Check the compiler's suggestions alongside your project's dependency versions.

### Why does the compiler say `terminal` does not need `mut`?

The current example uses:

```rust
ratatui::run(|terminal| {
    // ...
})
```

Do not add an extra `mut` to this closure parameter.

### How do I quit?

Press `q`, `Q`, or `Esc`. Ratatui restores the terminal state on normal exit.
