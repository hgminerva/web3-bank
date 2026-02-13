# Bank Contract

This is a simple **ink! smart contract** for managing a bank ledger on Substrate-based blockchains.  
It allows setting up a bank, managing accounts, and tracking balances, with events for all important actions.

---

## Features

- Create a new bank with an asset ID, manager, and maximum accounts.
- Deposit, withdraw, debit, and credit accounts (ledger management not fully implemented in this snippet).
- Emit events for all transactions.
- Supports **custom error messages** and **success messages** for transaction tracking.
- Bank status tracking: Open/Close, Frozen/Liquid accounts.
- Fully compatible with **WASM target** (`wasm32-unknown-unknown`) for Substrate contracts.

---

## Contract Structure

- `Bank`: Main storage struct containing bank metadata and ledgers.
- `Ledger`: Represents an individual account with balance and status.
- `Error`: Enum with possible error messages (e.g., `AccountNotFound`, `AccountBalanceInsufficient`).
- `Success`: Enum with transaction success messages.
- `BankTransactionStatus`: Combines success and error for events.
- `BankingEvent`: Event emitted on every transaction or setup.

---

## Constructors

- `new(asset_id: u128, maximum_accounts: u16) -> Self`  
  Creates a new bank instance. Sets the caller as the owner and manager.

- `default() -> Self`  
  Default bank setup (asset ID = 0, max accounts = 0).

---

## Messages

- `setup(asset_id: u128, manager: AccountId, maximum_accounts: u16) -> Result<(), Error>`  
  Configures the bank. Can only be called by the owner.

- `get() -> (u128, AccountId, AccountId, u16, u8)`  
  Returns bank information: `(asset_id, owner, manager, maximum_accounts, status)`.

---

## Events

All operations emit a `BankingEvent` with:

- `operator`: The caller account
- `status`: The transaction status (success or error)

---

## Building

Make sure you have the **WASM target** and `rust-src` installed:

```bash
rustup toolchain install 1.89
rustup override set 1.89
rustup component add rust-src --toolchain 1.89
rustup target add wasm32-unknown-unknown --toolchain 1.89
cargo install --force --locked cargo-contract --tag v3.2.0 --git https://github.com/use-ink/cargo-contract
cargo contract build --release