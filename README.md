# Bank Contract (ink!)

A simple on-chain bank contract built with **ink!**. Supports **deposit, withdraw, debit, and credit operations**, with manager-controlled and account-specific permissions.

## Features

* Create a new bank with `asset_id`, `manager`, and `maximum_accounts`.
* Open or close the bank.
* Deposit and withdraw assets from accounts.
* Credit (add) to an account — manager only.
* Debit (deduct) from an account — account holder only.
* Events emitted for **success** and **error** conditions.
* Checks for bank open/close, liquidity, account existence, and balance overflow/insufficient errors.

---

## Storage

```rust
struct Bank {
    asset_id: u128,           // Bank asset identifier
    owner: AccountId,         // Bank owner
    manager: AccountId,       // Bank manager
    maximum_accounts: u16,    // Max ledger accounts
    ledgers: Vec<Ledger>,     // Accounts ledger
    status: u8,               // Bank status: 0 = Open, 1 = Close
}

struct Ledger {
    account: AccountId,       // Account address
    balance: u128,            // Free balance
    status: u8,               // Account status: 0 = Frozen, 1 = Liquid
}
```

---

## Error Messages

```rust
enum Error {
    BadOrigin,
    BankIsClose,
    BankAccountMaxOut,
    AccountAlreadyExist,
    AccountNotFound,
    AccountBalanceInsufficient,
    AccountBalanceOverflow,
    AccountFrozen,
}
```

---

## Success Messages

```rust
enum Success {
    BankSetupSuccess,
    BankCloseSuccess,
    BankOpenSuccess,
    AccountDepositSuccess,
    AccountWithdrawalSuccess,
    AccountDebitSuccess,
    AccountCreditSuccess,
}
```

---

## Events

```rust
#[ink(event)]
struct BankingEvent {
    operator: AccountId,
    status: BankTransactionStatus,
}

enum BankTransactionStatus {
    EmitSuccess(Success),
    EmitError(Error),
}
```

Events are emitted for **every transaction**, indicating **success or error status**.

---

## Constructors

### `new`

```rust
pub fn new(asset_id: u128, maximum_accounts: u16) -> Self
```

Creates a new bank. The caller becomes the owner and manager.

### `default`

```rust
pub fn default() -> Self
```

Creates a bank with default parameters (`asset_id = 0`, `maximum_accounts = 0`).

---

## Bank Control

### `setup`

```rust
pub fn setup(asset_id: u128, manager: AccountId, maximum_accounts: u16) -> Result<(), Error>
```

Only the **owner** can call. Resets ledgers and updates bank info.

### `open`

```rust
pub fn open() -> Result<(), Error>
```

Only the **manager** can open the bank. Sets `status = 0`.

### `close`

```rust
pub fn close() -> Result<(), Error>
```

Only the **manager** can close the bank. Sets `status = 1`.

---

## Account Operations

### `deposit`

```rust
pub fn deposit(account: AccountId, amount: u128) -> Result<(), Error>
```

* Only **manager** can deposit.
* Adds to existing ledger or creates a new account if space allows.
* Checks for bank status and balance overflow.

### `withdraw`

```rust
pub fn withdraw(account: AccountId, amount: u128) -> Result<(), Error>
```

* Only **manager** can withdraw from accounts.
* Checks bank open status and sufficient balance.

### `credit`

```rust
pub fn credit(account: AccountId, amount: u128) -> Result<(), Error>
```

* Only **manager** can credit an account.
* Adds to account balance.
* Checks account liquidity and overflow.

### `debit`

```rust
pub fn debit(amount: u128) -> Result<(), Error>
```

* Account **owner only** can debit their own balance.
* Deducts from balance if sufficient and account is liquid.

---

## Read Operations

### `get`

```rust
pub fn get() -> (u128, AccountId, AccountId, u16, u8)
```

Returns the bank information:

* `asset_id`
* `owner`
* `manager`
* `maximum_accounts`
* `status`

---

## Events Example

```rust
BankingEvent {
    operator: caller,
    status: BankTransactionStatus::EmitSuccess(Success::AccountDepositSuccess),
}
```

All operations emit either `EmitSuccess` or `EmitError` for **easy tracking**.

---

## Notes

* All operations are **borrow-safe** in Rust — no E0502 errors.
* Credit/debit operations respect **liquidity (frozen/liquid) status**.
* Deposit/withdraw/checks enforce **bank open/close** rules.

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