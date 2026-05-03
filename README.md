# Bank Contract (ink!)

An on-chain bank contract built with **ink!**. Supports **deposit, withdraw, debit, credit, interest crediting, loan application, loan payment, and loan liquidation**, with manager-controlled and account-specific permissions.

---

## Features

* Create a new bank with `asset_id`, `loan_asset_id`, `manager`, `maximum_accounts`, `threshold`, and `daily_blocks`.
* Open or close the bank (manager only).
* Deposit and withdraw assets from accounts (manager only).
* Credit (add) or debit (deduct) account balances.
* Credit interest across all liquid accounts based on their average daily balance (ADB).
* Apply for a collateral-backed loan (manager only, requires oracle price input).
* Make partial or full loan payments; collateral is returned on full repayment.
* Liquidate under-collateralized loans based on a price oracle feed.
* Events emitted for **every success and error condition**.
* Average Daily Balance (ADB) computed on every deposit, withdrawal, credit, and debit.

---

## Storage

```rust
struct Bank {
    asset_id: u128,           // Bank deposit asset identifier
    loan_asset_id: u128,      // Bank loan asset identifier
    owner: AccountId,         // Bank owner
    manager: AccountId,       // Bank manager
    maximum_accounts: u16,    // Max ledger accounts
    daily_blocks: u16,        // Number of blocks per day (used for ADB)
    threshold: u16,           // Loan liquidation threshold in percentage
    ledgers: Vec<Ledger>,     // Account ledger
    loans: Vec<Loan>,         // Active loans
    status: u8,               // Bank status: 0 = Open, 1 = Close
}

struct Ledger {
    account: AccountId,         // Account address
    balance: u128,              // Free balance
    adb: u128,                  // Average daily balance
    adb_beginning_block: u128,  // Block when ADB tracking started
    status: u8,                 // 0 = Frozen, 1 = Liquid
}

struct Loan {
    account: AccountId,       // Borrower account
    collateral: u128,         // Collateral amount locked from ledger
    loan_amount: u128,        // Original loan amount
    paid_amount: u128,        // Total amount paid so far
    balance: u128,            // Remaining balance: loan_amount - paid_amount
    liquidation_price: u128,  // Price at which the loan is liquidated
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
    LoanNotFound,
    LoanAlreadyExist,
    LoanCollateralInsufficient,
    LoanComputationOverflow,
    ExcessivePayment,
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
    LoanApplicationSuccess,
    LoanFullyPaidSuccess,
    LoanPaymentSuccess,
    LoanLiquidationSuccess,
}
```

---

## Events

```rust
#[ink(event)]
struct BankingEvent {
    #[ink(topic)]
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
pub fn new(
    asset_id: u128,
    loan_asset_id: u128,
    maximum_accounts: u16,
    threshold: u16,
    daily_blocks: u16,
) -> Self
```

Creates a new bank. The caller becomes both the owner and manager.

### `default`

```rust
pub fn default() -> Self
```

Creates a bank with default parameters (`asset_id = 0`, `loan_asset_id = 0`, `maximum_accounts = 0`, `threshold = 0`, `daily_blocks = 1`).

---

## Bank Control

### `setup`

```rust
pub fn setup(
    asset_id: u128,
    loan_asset_id: u128,
    manager: AccountId,
    maximum_accounts: u16,
    threshold: u16,
    daily_blocks: u16,
) -> Result<(), Error>
```

Only the **owner** can call. Resets all ledgers and loans, then updates bank configuration.

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
* Adds to an existing ledger balance or creates a new account if space allows.
* Checks for bank open status, maximum accounts, and balance overflow.
* Recomputes the account's **ADB** on every deposit.

### `withdraw`

```rust
pub fn withdraw(account: AccountId, amount: u128) -> Result<(), ContractError>
```

* Only **manager** can withdraw.
* Checks bank open status and sufficient balance.
* Calls the asset pallet runtime to transfer funds back to the account.
* Recomputes the account's **ADB** on every withdrawal.

### `credit`

```rust
pub fn credit(account: AccountId, amount: u128) -> Result<(), Error>
```

* Only **manager** can credit an account.
* Adds to account balance.
* Checks account liquidity and balance overflow.
* Recomputes **ADB**.

### `debit`

```rust
pub fn debit(amount: u128) -> Result<(), Error>
```

* Account **owner only** can debit their own balance.
* Deducts from balance if sufficient and account is liquid.
* Recomputes **ADB**.

### `credit_interest`

```rust
pub fn credit_interest(rate: u128) -> Result<(), Error>
```

* Only **manager** can call.
* Loops through all **liquid** ledger accounts and credits interest computed as:
  ```
  interest = adb * rate / 100
  ```
* Recomputes **ADB** after crediting interest to each account.

---

## Loan Operations

### `loan_application`

```rust
pub fn loan_application(
    account: AccountId,
    loan_amount: u128,
    price: u128,
    collateral: u128,
) -> Result<(), Error>
```

* Only **manager** can call (requires oracle price input).
* Validates that the account exists and is not frozen.
* Validates that the account balance covers the collateral.
* Validates that the collateral value at the threshold price covers the loan amount:
  ```
  threshold_price = price + (price * threshold / 100)
  collateral_value = collateral * threshold_price
  loan_amount <= collateral_value
  ```
* Rejects if a loan for the account already exists.
* Computes the liquidation price:
  ```
  liquidation_price = (loan_amount + loan_amount * threshold / 100) / collateral
  ```
* Pushes a new `Loan` entry into the loans vector.

### `loan_payment`

```rust
pub fn loan_payment(account: AccountId, amount: u128) -> Result<(), Error>
```

* Only **manager** can call (after accepting the loan asset transfer off-chain).
* Looks up the loan by account in the loans vector.
* **Full payment** (`amount >= balance`): removes the loan and adds the collateral back to the account's ledger balance.
* **Partial payment** (`amount < balance`): safely increments `paid_amount` and recomputes `balance`:
  ```
  paid_amount = paid_amount + amount   // checked_add
  balance     = loan_amount - paid_amount
  ```

### `loan_liquidation`

```rust
pub fn loan_liquidation(price: u128) -> Result<(), Error>
```

* Only **manager** can call (triggered by oracle price feed).
* Scans all active loans and collects those where `liquidation_price >= price`.
* Removes all identified loans in **reverse index order** to preserve correct indices during removal.
* Collateral is **forfeited** on liquidation and is not returned to the account.

---

## Read Operations

### `get`

```rust
pub fn get() -> (u128, AccountId, AccountId, u16, u16, u16, u8)
```

Returns the bank information as a tuple:

| Field | Type |
|---|---|
| `asset_id` | `u128` |
| `owner` | `AccountId` |
| `manager` | `AccountId` |
| `maximum_accounts` | `u16` |
| `threshold` | `u16` |
| `daily_blocks` | `u16` |
| `status` | `u8` |

### `get_balance`

```rust
pub fn get_balance(account: AccountId) -> Option<Ledger>
```

Returns the full `Ledger` struct for a given account, or `None` if not found.

---

## Average Daily Balance (ADB)

The ADB is a time-weighted balance recomputed on every transaction:

```
blocks_elapsed = current_block - adb_beginning_block
adb = balance * blocks_elapsed / daily_blocks
```

`adb_beginning_block` is set when the ledger account is first created and remains fixed. The ADB is used by `credit_interest` to compute proportional interest for each account.

---

## Loan Liquidation Formula

```
liquidation_price = (loan_amount + loan_amount * threshold / 100) / collateral
```

When the oracle price drops to or below `liquidation_price`, the loan is liquidated and removed. The collateral is forfeited.

---

## Events Example

```rust
BankingEvent {
    operator: caller,
    status: BankTransactionStatus::EmitSuccess(Success::AccountDepositSuccess),
}
```

All operations emit either `EmitSuccess` or `EmitError` for **easy on-chain tracking**.

---

## Notes

* All arithmetic uses **`checked_add`, `checked_sub`, `checked_mul`, `checked_div`** with `ok_or(Error::LoanComputationOverflow)` or `ok_or(Error::AccountBalanceOverflow)` to prevent panics.
* Loan vector removals during liquidation use **reverse-order iteration** to avoid index shifting bugs.
* On full loan repayment, the locked **collateral is returned** to the borrower's ledger balance.
* On liquidation, collateral is **forfeited** — it is not returned.
* Credit/debit operations respect **account liquidity (frozen/liquid) status**.
* Deposit/withdraw/loan operations enforce the **bank open/close** rule.

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
```