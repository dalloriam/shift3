build +args='':
    cargo build {{args}}

_clippy target:
    @cd {{target}} && cargo check && cargo clippy

lint:
    @just _clippy action-executor
    @just _clippy gcloud
    @just _clippy process
    @just _clippy protocol
    @just _clippy toolkit
    @just _clippy trigger-interpreter
    @just _clippy trigger-system

doc target +args='':
    cargo doc -p {{target}} {{args}}
