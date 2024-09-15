use alloy::node_bindings::Anvil;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let anvil = Anvil::new().fork("https://mainnet.base.org").try_spawn()?;

    println!("Anvil running at `{}`", anvil.endpoint());

    Ok(())
}
