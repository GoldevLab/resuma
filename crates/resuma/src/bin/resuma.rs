//! Global CLI binary shipped with the `resuma` crate (`cargo install resuma`).

fn main() -> anyhow::Result<()> {
    resuma_cli::run()
}
