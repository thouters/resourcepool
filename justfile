
serve:
    cargo run --bin server -- -c tests/simple_inventory.yaml serve
test:
    cargo test -- --nocapture
