# Adding and Removing

In this tutorial, we'll teach you step by step process to migrate a cosmwasm contract to the secret network.

## Step 1

Copy the `cw-counter` folder and rename it `secret-counter`

## Multi-Tests

When working with Secret Network, it's crucial to understand that official support for multi-tests is not provided. As a result, it's advisable to exclude the `integration_tests.rs` file from your project. To implement this, you'll need to...

### 1. Remove specific lines from your `Cargo.toml` file

```toml
[dev-dependencies]
cw-multi-test = "0.15.1"
```

### 2. Remove/Delete the `integration_tests.rs` file

Next, it's best to remove `integration_tests.rs` because as I mentioned before secret network doesn't support multi-tests.


### 3. Remove this line from `lib.rs` as well

```Rust
pub mod integration_tests;
```

### 4. Remove/Delete the `helpers.rs` file

Since in this example `helper.rs` are only used in `integration_tests.rs`

### 5. Remove this line from `lib.rs` as well

```Rust
pub mod helpers;
```

NOTE: There are some other teams that have worked with multi-test. [This](https://github.com/securesecrets/secret-plus-utils) package by secure secrets can be an alternative to secret networks storage package aka secret-toolkit. But it's recommended to use secret-toolkit for storage packages. There're other tools available on the secret as well that offer multi-contract testing like Fadroma. [Fadroma](https://fadroma.tech/guide.html) is an application framework for the CosmWasm Compute module. Fadroma includes Rust libraries for writing smart contracts and a TypeScript system for building, deploying, and interacting with them.
