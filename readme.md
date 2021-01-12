# git file-url

This simple tool allows you to get a file git url

usage :

```bash
    ❯ git file-url Cargo.toml
    https://github.com/nbouliol/git-file-url/blob/master/Cargo.toml
```

install :

Make sure [rustup](https://rustup.rs/) is installed

Then

```bash
    git clone https://github.com/nbouliol/git-file-url
    cd git-file-url
    cargo build --release
```

The binary will be available in the `target/release` directory.
