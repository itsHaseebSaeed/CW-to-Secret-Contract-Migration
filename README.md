# Migrating CW Counter Contract to Secret

This repository contains a Cosmwasm smart contract that allows for a simple incrementing counter. We'll convert this contract to Secret network compatible contract.

The contract has been written in Rust and uses Secret Network's CosmWasm module. The core functionality of the contract is to increment a count and maintain an individual count for each user.

In addition to the basic incrementing counter, we'll also add a viewing key functionality, enabling privacy features by ensuring only users with the correct viewing keys can query the count for a given user.

Features

1. Incrementing Counter: The core function of the contract, increment, allows a user to increment the global counter by one.

2. User Counter: The contract also maintains an individual counter for each user that invokes the increment function.

3. Viewing Key: The contract implements viewing keys functionality, ensuring that only users with the correct viewing keys can query the count for a given user.

4. Query Permit: The contract implements query permit functionality, ensuring that only authorized user can query the count for a given user.

Contributing
Contributions to this project are welcome. Please submit a pull request with your changes or improvements.

License
This project is licensed under the terms of the MIT license
