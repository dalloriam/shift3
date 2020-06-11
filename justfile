build +args='':
    cargo build {{args}}

check +args='':
    cargo check {{args}}
    cargo clippy {{args}}
    cargo test {{args}}

doc target +args='':
    cargo doc -p {{target}} {{args}}