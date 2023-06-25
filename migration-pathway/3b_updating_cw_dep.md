# Updating CosmWasm Dependencies

While working on the Secret Network, you'll find that you need to swap out certain existing CosmWasm dependencies, including `cosmwasm-schema`, `cosmwasm-std`, and `cosmwasm-storage`. The Secret Network has custom versions of these packages that are designed to work seamlessly with its ecosystem.

Kick-start this process by navigating to your `Cargo.toml` file and removing the following lines:

```toml
[dependencies]
cosmwasm-schema = "1.1.3"
cosmwasm-std = "1.1.3"
cosmwasm-storage = "1.1.3"
```

Once you've removed the aforementioned lines, replace them with:

```toml
cosmwasm-schema = { version = "1.1.8" }
cosmwasm-std = { git = "https://github.com/scrtlabs/cosmwasm/", default-features = false, tag = "v1.1.9-secret" }
cosmwasm-storage = { git = "https://github.com/scrtlabs/cosmwasm/", tag = "v1.1.9-secret" }
secret-toolkit = { git = "https://github.com/scrtlabs/secret-toolkit", features = [
  "permit",
  "viewing-key",
], rev = "9b74bdac71c2fedcc12246f18cdfdd94b8991282" }
```

Upon making these alterations to your dependencies, you might run into errors in files like `contract.rs`, `helpers.rs`, and `state.rs`. Don't worry, this is expected because of the unique requirements of the Secret Network's versions of the CosmWasm packages. We'll be addressing these errors as we proceed further.
