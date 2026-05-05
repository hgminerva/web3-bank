#![cfg_attr(not(feature = "std"), no_std, no_main)]

/// pallet_assets runtime calls
pub mod assets;

/// Errors
pub mod errors;

#[ink::contract]
mod bank {

    use ink::prelude::vec::Vec;

    use crate::errors::{Error, RuntimeError, ContractError};
    use crate::assets::{AssetsCall, RuntimeCall};

    /// Success Messages
    #[derive(scale::Encode, scale::Decode, Debug, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Success {
        /// Bank setup successful
        BankSetupSuccess,
        /// Bank close successful
        BankCloseSuccess,
        /// Bank open successful
        BankOpenSuccess,
        /// Account deposit successful
        AccountDepositSuccess,
        /// Account withdrawal successful
        AccountWithdrawalSuccess,
        /// Account debit success
        AccountDebitSuccess,
        /// Account credit success
        AccountCreditSuccess,  
        /// Loan application success      
        LoanApplicationSuccess,
        /// Loan fully paid
        LoanFullyPaidSuccess,
        /// Loan payment success
        LoanPaymentSuccess,
        /// Loan liquidation success
        LoanLiquidationSuccess,
    }    

    /// Bank transaction status
    #[derive(scale::Encode, scale::Decode, Debug, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum BankTransactionStatus {
        EmitSuccess(Success),
        EmitError(Error),
    }    

    /// Bank events
    #[ink(event)]
    pub struct BankingEvent {
        #[ink(topic)]
        operator: AccountId,
        status: BankTransactionStatus,
    }     

    /// Bank ledger
    #[derive(scale::Encode, scale::Decode, Clone, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct Ledger {
        /// Account address
        pub account: AccountId,
        /// Free balance
        pub balance: u128,
        /// Average daily balance.  Computed every incoming and outgoing transactions
        pub adb: u128,
        /// Average daily balance beginning block.  Sets upon creation of the ledger.
        /// This is used to compute for the time-weighted average balance:
        ///    adb = (balance x [current_block - adb_beginning_block]) / bank.daily_blocks
        pub adb_beginning_block: u128,
        /// Status (0-Frozen, 1-Liquid)
        pub status: u8,
    }        

    #[derive(scale::Encode, scale::Decode, Clone, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct Loan {
        /// Account address
        pub account: AccountId,
        /// Collateral
        pub collateral: u128,
        /// Loan amount
        pub loan_amount: u128,
        /// Paid amount
        pub paid_amount: u128,
        /// Computed: loan_amount - paid_amount
        pub balance: u128,        
        /// Liquidation price: (balance × threshold) / (collateral × 100)
        pub liquidation_price: u128,
    }    

    /// Bank storage
    #[ink(storage)]
    pub struct Bank {
        /// Bank asset
        pub asset_id: u128,
        /// Bank loan asset
        pub loan_asset_id: u128,
        /// Bank owner
        pub owner: AccountId,
        /// Bank manager
        pub manager: AccountId,
        /// Maximum accounts the bank ledger can handle
        pub maximum_accounts: u16,
        /// Daily blocks
        pub daily_blocks: u16,
        /// Threshold (loan price threshold in percentage)
        pub threshold: u16,
        /// Bank ledgers
        pub ledgers: Vec<Ledger>,
        /// Bank loans
        pub loans: Vec<Loan>,
        /// Status (0-Open, 1-Close)
        pub status: u8,
    }

    impl Bank {

        /// Create new bank
        #[ink(constructor)]
        pub fn new(asset_id: u128, 
            loan_asset_id: u128,
            maximum_accounts: u16,
            threshold: u16,
            daily_blocks: u16) -> Self {

            let caller: ink::primitives::AccountId = Self::env().caller();

            Self { 
                asset_id: asset_id, 
                loan_asset_id: loan_asset_id,
                owner: caller,
                manager: caller,
                maximum_accounts: maximum_accounts,
                threshold: threshold,
                ledgers: Vec::new(),
                loans: Vec::new(),
                daily_blocks: daily_blocks,
                status: 0u8,
            }
        }

        /// Default setup
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(0u128, 0u128, 0u16, 0u16, 1u16)
        }

        /// Setup bank
        #[ink(message)]
        pub fn setup(&mut self,
            asset_id: u128,
            loan_asset_id: u128,
            manager: AccountId,
            maximum_accounts: u16,
            threshold: u16,
            daily_blocks: u16) -> Result<(), Error> {
            
            // Setup can only be done by the owner
            let caller = self.env().caller();
            if self.env().caller() != self.owner {
                self.env().emit_event(BankingEvent {
                    operator: caller,
                    status: BankTransactionStatus::EmitError(Error::BadOrigin),
                });
                return Ok(());
            } 

            // The setup will delete all existing accounts - Very Important!
            self.asset_id = asset_id;
            self.loan_asset_id = loan_asset_id;
            self.manager = manager;
            self.maximum_accounts = maximum_accounts;
            self.threshold = threshold;
            self.ledgers =  Vec::new();
            self.loans =  Vec::new();
            self.daily_blocks = daily_blocks;
            self.status = 0;

            self.env().emit_event(BankingEvent {
                operator: caller,
                status: BankTransactionStatus::EmitSuccess(Success::BankSetupSuccess),
            });

            Ok(())
        }

        /// Get the bank information
        #[ink(message)]
        pub fn get(&self) -> (u128, AccountId, AccountId, u16, u16, u16, u8) {
            (
                self.asset_id,
                self.owner,
                self.manager,
                self.maximum_accounts,
                self.threshold,
                self.daily_blocks,
                self.status,
            )
        }

        /// Close the bank
        #[ink(message)]
        pub fn close(&mut self) -> Result<(), Error> {

            // Closing the can only be done by the manager
            let caller = self.env().caller();
            if self.env().caller() != self.manager {
                self.env().emit_event(BankingEvent {
                    operator: caller,
                    status: BankTransactionStatus::EmitError(Error::BadOrigin),
                });
                return Ok(());
            } 

            // This will close the bank
            self.status = 1;

            self.env().emit_event(BankingEvent {
                operator: caller,
                status: BankTransactionStatus::EmitSuccess(Success::BankCloseSuccess),
            });

            Ok(())
        }

        /// Open the bank
        #[ink(message)]
        pub fn open(&mut self) -> Result<(), Error> {

            // Opening the can only be done by the manager
            let caller = self.env().caller();
            if self.env().caller() != self.manager {
                self.env().emit_event(BankingEvent {
                    operator: caller,
                    status: BankTransactionStatus::EmitError(Error::BadOrigin),
                });
                return Ok(());
            } 

            // This will open the bank
            self.status = 0;

            self.env().emit_event(BankingEvent {
                operator: caller,
                status: BankTransactionStatus::EmitSuccess(Success::BankOpenSuccess),
            });

            Ok(())
        }        

        /// Deposit to the bank
        #[ink(message)]
        pub fn deposit(&mut self,
            account: AccountId,
            amount: u128) -> Result<(), Error> {

            let current_block = self.env().block_number() as u128;

            // Deposit can only be done by the manager once the transfer of the 
            // asset is verified through the tx-hash.
            let caller = self.env().caller();
            if self.env().caller() != self.manager {
                self.env().emit_event(BankingEvent {
                    operator: caller,
                    status: BankTransactionStatus::EmitError(Error::BadOrigin),
                });
                return Ok(());
            } 

            // Check if the bank is open
            if self.status != 0 {
                self.env().emit_event(BankingEvent {
                    operator: caller,
                    status: BankTransactionStatus::EmitError(Error::BankIsClose),
                });
                return Ok(());
            }

            // Search if the account exist already, if it does in just add to the
            // ledger the amount deposited, if not then create the new account.
            // 1. Update a balance
            let mut account_found = false;
            for ledger in self.ledgers.iter_mut() {
                if ledger.account == account {
                    
                    ledger.balance = ledger
                        .balance
                        .checked_add(amount)
                        .ok_or(Error::AccountBalanceOverflow)?; 

                    // ADB computation
                    let blocks_elapsed = current_block
                        .saturating_sub(ledger.adb_beginning_block);

                    ledger.adb = ledger.balance
                        .checked_mul(blocks_elapsed)
                        .ok_or(Error::AccountBalanceOverflow)?
                        .checked_div(self.daily_blocks.into())
                        .unwrap_or(0);

                    account_found = true;
                    break;
                }
            }
            // 2. Create a new account if the account does not exist
            if !account_found {
                if self.ledgers.len() as u16 >= self.maximum_accounts {
                    self.env().emit_event(BankingEvent {
                        operator: caller,
                        status: BankTransactionStatus::EmitError(Error::BankAccountMaxOut),
                    });
                    return Ok(());
                }
                let new_ledger = Ledger {
                    account,
                    balance: amount,
                    adb: amount,
                    adb_beginning_block: current_block,
                    status: 1, // 1 = Liquid
                };
                self.ledgers.push(new_ledger);
            }

            self.env().emit_event(BankingEvent {
                operator: caller,
                status: BankTransactionStatus::EmitSuccess(Success::AccountDepositSuccess),
            });

            Ok(())
        }

        /// Withdraw from the bank
        #[ink(message)]
        pub fn withdraw(&mut self,
            account: AccountId,
            amount: u128) -> Result<(), ContractError> {

            let current_block = self.env().block_number() as u128;

            // Withdraw can only be done by the manager once the balance of the account
            // is sufficient for withdrawal
            let caller = self.env().caller();
            if self.env().caller() != self.manager {
                self.env().emit_event(BankingEvent {
                    operator: caller,
                    status: BankTransactionStatus::EmitError(Error::BadOrigin),
                });
                return Ok(());
            } 

            // Check if the bank is open
            if self.status != 0 {
                self.env().emit_event(BankingEvent {
                    operator: caller,
                    status: BankTransactionStatus::EmitError(Error::BankIsClose),
                });
                return Ok(());
            }

            // Search if the account exist already, if it does, check if the balance is
            // sufficient, if so, deduct the ledger, if not raise a balance insufficient
            // error.
            let mut account_found = false;
            for ledger in self.ledgers.iter_mut() {
                if ledger.account == account {
                    account_found = true;

                    // Check if balance is sufficient
                    if ledger.balance < amount {
                        self.env().emit_event(BankingEvent {
                            operator: caller,
                            status: BankTransactionStatus::EmitError(Error::AccountBalanceInsufficient),
                        });
                        return Ok(());
                    }

                    // Deduct the amount
                    ledger.balance -= amount;

                    // ADB computation
                    let blocks_elapsed = current_block
                        .saturating_sub(ledger.adb_beginning_block);

                    ledger.adb = ledger.balance
                        .checked_mul(blocks_elapsed)
                        .ok_or(Error::AccountBalanceOverflow)?
                        .checked_div(self.daily_blocks.into())
                        .unwrap_or(0);                    

                    // Transfer the asset to the account
                    self.env()
                        .call_runtime(&RuntimeCall::Assets(AssetsCall::Transfer {
                            id: self.asset_id,
                            target: account.into(),
                            amount: amount,
                        }))
                        .map_err(|_| RuntimeError::CallRuntimeFailed)?;

                    break;
                }
            }

            if !account_found {
                self.env().emit_event(BankingEvent {
                    operator: caller,
                    status: BankTransactionStatus::EmitError(Error::AccountNotFound),
                });
                return Ok(());
            }

            self.env().emit_event(BankingEvent {
                operator: caller,
                status: BankTransactionStatus::EmitSuccess(Success::AccountWithdrawalSuccess),
            });

            Ok(())
        }

        /// Credit to the account (add).  This is done by the manager only.
        #[ink(message)]
        pub fn credit(&mut self,
            account: AccountId,
            amount: u128) -> Result<(), Error> {
            
            let current_block = self.env().block_number() as u128;

            // Credit is adding to the balance of an account, this is done only
            // by the manager.
            let caller = self.env().caller();

            if self.env().caller() != self.manager {
                self.env().emit_event(BankingEvent {
                    operator: caller,
                    status: BankTransactionStatus::EmitError(Error::BadOrigin),
                });
                return Ok(());
            } 

            // Check if the bank is open
            if self.status != 0 {
                self.env().emit_event(BankingEvent {
                    operator: caller,
                    status: BankTransactionStatus::EmitError(Error::BankIsClose),
                });
                return Ok(());
            }

            // Search for the caller account in the ledger, if found, add to the balance
            // the given amount.
            let mut account_found = false;

            for ledger in self.ledgers.iter_mut() {
                if ledger.account == account {
                    account_found = true;

                    // Check if account is liquid
                    if ledger.status != 1 {
                        self.env().emit_event(BankingEvent {
                            operator: caller,
                            status: BankTransactionStatus::EmitError(Error::AccountFrozen),
                        });
                        return Ok(());
                    }

                    // Add the amount to the balance safely
                    match ledger.balance.checked_add(amount) {
                        Some(new_balance) => {
                            ledger.balance = new_balance;

                            // ADB computation
                            let blocks_elapsed = current_block
                                .saturating_sub(ledger.adb_beginning_block);

                            ledger.adb = ledger.balance
                                .checked_mul(blocks_elapsed)
                                .ok_or(Error::AccountBalanceOverflow)?
                                .checked_div(self.daily_blocks.into())
                                .unwrap_or(0);  
                        },
                        None => {
                            self.env().emit_event(BankingEvent {
                                operator: caller,
                                status: BankTransactionStatus::EmitError(Error::AccountBalanceOverflow),
                            });
                            return Ok(());
                        }
                    }

                    break;
                }
            }

            if !account_found {
                self.env().emit_event(BankingEvent {
                    operator: caller,
                    status: BankTransactionStatus::EmitError(Error::AccountNotFound),
                });
                return Ok(());
            }

            self.env().emit_event(BankingEvent {
                operator: caller,
                status: BankTransactionStatus::EmitSuccess(Success::AccountCreditSuccess),
            });

            Ok(())
        }

        /// Debit to the account (deduct).  This is done by any depositor.
        #[ink(message)]
        pub fn debit(&mut self,
            amount: u128) -> Result<(), Error> {

            let current_block = self.env().block_number() as u128;
            let caller = self.env().caller();

            // Check if the bank is open
            if self.status != 0 {
                self.env().emit_event(BankingEvent {
                    operator: caller,
                    status: BankTransactionStatus::EmitError(Error::BankIsClose),
                });
                return Ok(());
            }

            // Search for the caller account in the ledger
            let mut account_found = false;

            for ledger in self.ledgers.iter_mut() {
                if ledger.account == caller {
                    account_found = true;

                    // Check if account is liquid
                    if ledger.status != 1 {
                        self.env().emit_event(BankingEvent {
                            operator: caller,
                            status: BankTransactionStatus::EmitError(Error::AccountFrozen),
                        });
                        return Ok(());
                    }

                    // Check if balance is sufficient
                    if ledger.balance < amount {
                        self.env().emit_event(BankingEvent {
                            operator: caller,
                            status: BankTransactionStatus::EmitError(Error::AccountBalanceInsufficient),
                        });
                        return Ok(());
                    }

                    ledger.balance -= amount;

                    // ADB computation
                    let blocks_elapsed = current_block
                        .saturating_sub(ledger.adb_beginning_block);

                    ledger.adb = ledger.balance
                        .checked_mul(blocks_elapsed)
                        .ok_or(Error::AccountBalanceOverflow)?
                        .checked_div(self.daily_blocks.into())
                        .unwrap_or(0); 

                    break;
                }
            }

            // Account not found
            if !account_found {
                self.env().emit_event(BankingEvent {
                    operator: caller,
                    status: BankTransactionStatus::EmitError(Error::AccountNotFound),
                });
                return Ok(());
            }

            self.env().emit_event(BankingEvent {
                operator: caller,
                status: BankTransactionStatus::EmitSuccess(Success::AccountDebitSuccess),
            });

            Ok(())
        }

        /// Credit interest.  Interest are computed off-chain and needs a manual
        /// transfer to the bank smart contract. 
        #[ink(message)]
        pub fn credit_interest(&mut self,
            rate: u128) -> Result<(), Error> {

            let current_block = self.env().block_number() as u128;

            // Credit is adding to the balance of an account, this is done only
            // by the manager.
            let caller = self.env().caller();

            if self.env().caller() != self.manager {
                self.env().emit_event(BankingEvent {
                    operator: caller,
                    status: BankTransactionStatus::EmitError(Error::BadOrigin),
                });
                return Ok(());
            } 

            // Check if the bank is open
            if self.status != 0 {
                self.env().emit_event(BankingEvent {
                    operator: caller,
                    status: BankTransactionStatus::EmitError(Error::BankIsClose),
                });
                return Ok(());
            }

            for ledger in self.ledgers.iter_mut() {
                if ledger.status == 0 {
                    // Compute interest: interest = adb * rate / 100
                    let interest = ledger.adb
                        .checked_mul(rate)
                        .ok_or(Error::AccountBalanceOverflow)?
                        .checked_div(100)
                        .unwrap_or(0);

                    // Credit interest to balance
                    ledger.balance = ledger.balance
                        .checked_add(interest)
                        .ok_or(Error::AccountBalanceOverflow)?;

                    // ADB computation
                    let blocks_elapsed = current_block
                        .saturating_sub(ledger.adb_beginning_block);

                    ledger.adb = ledger.balance
                        .checked_mul(blocks_elapsed)
                        .ok_or(Error::AccountBalanceOverflow)?
                        .checked_div(self.daily_blocks.into())
                        .unwrap_or(0);
                }
            }

            self.env().emit_event(BankingEvent {
                operator: caller,
                status: BankTransactionStatus::EmitSuccess(Success::AccountCreditSuccess),
            });

            Ok(())
        }

        /// Apply for a loan
        /// For example: loan_amount (encoded) = $100 USDT
        ///              price (oracle) = $0.01
        ///              collateral (encoded) = 11,000 
        ///              threshold (setup) = 5% (Upon liquidation the value must be $105)
        ///              liquidation_price (computed) = $105 / 11,000 = $0.00954545454
        /// Rules:
        ///     1. The account must have a balance greater than the collateral.
        ///     2. The collateral must be within the threshold.
        ///     3. To have an acceptable liquidation_price the collateral must take into consideration the 
        ///        volatility of the asset price or else the loan will immediately liquidated.
        #[ink(message)]
        pub fn loan_application(&mut self,
            account: AccountId,
            loan_amount: u128,
            price: u128,
            collateral: u128) -> Result<(), Error> {

            // Loan application can only be called by the manager due to oracle input
            let caller = self.env().caller();
            if self.env().caller() != self.manager {
                self.env().emit_event(BankingEvent {
                    operator: caller,
                    status: BankTransactionStatus::EmitError(Error::BadOrigin),
                });
                return Ok(());
            } 

            // Check if the bank is open
            if self.status != 0 {
                self.env().emit_event(BankingEvent {
                    operator: caller,
                    status: BankTransactionStatus::EmitError(Error::BankIsClose),
                });
                return Ok(());
            } 

            // Check if the account is a depositor
            let ledger_index = self.ledgers.iter().position(|l| l.account == account);
            let ledger_index = match ledger_index {
                Some(i) => i,
                None => {
                    self.env().emit_event(BankingEvent {
                        operator: caller,
                        status: BankTransactionStatus::EmitError(Error::AccountNotFound),
                    });
                    return Ok(());
                }
            };

            // Check if the account is frozen
            if self.ledgers[ledger_index].status == 1 {
                self.env().emit_event(BankingEvent {
                    operator: caller,
                    status: BankTransactionStatus::EmitError(Error::AccountFrozen),
                });
                return Ok(());
            }

            // Check if the balance can cover the collateral
            if collateral > self.ledgers[ledger_index].balance {
                self.env().emit_event(BankingEvent {
                    operator: caller,
                    status: BankTransactionStatus::EmitError(Error::AccountBalanceInsufficient),
                });
                return Ok(());
            }

            // Check if the collateral can cover the liquidation price
            // 1. compute for the threshold price: current price plus the threshold.
            // 2. use the threshold price to compute for the collateral value
            let threshold_price = price
                .checked_add(
                    price
                        .checked_mul(self.threshold.into())
                        .ok_or(Error::LoanComputationOverflow)?
                        .checked_div(100)
                        .ok_or(Error::LoanComputationOverflow)?
                )
                .ok_or(Error::LoanComputationOverflow)?;

            let collateral_value = collateral
                .checked_mul(threshold_price)
                .ok_or(Error::LoanComputationOverflow)?;

            if loan_amount > collateral_value {
                self.env().emit_event(BankingEvent {
                    operator: caller,
                    status: BankTransactionStatus::EmitError(Error::LoanCollateralInsufficient),
                });
                return Ok(());
            }

            // Check if there is an existing loan, if there is none then add a loan
            let loan_exists = self.loans.iter().any(|l| l.account == account);
            if loan_exists {
                self.env().emit_event(BankingEvent {
                    operator: caller,
                    status: BankTransactionStatus::EmitError(Error::LoanAlreadyExist),
                });
                return Ok(());
            }

            // Now add the loan, but first compute for the liquidation price.
            //      liquidation_price = (loan + threshold) / collateral
            let loan_with_threshold = loan_amount
                .checked_add(
                    loan_amount
                        .checked_mul(self.threshold as u128)
                        .ok_or(Error::LoanComputationOverflow)?
                        .checked_div(100)
                        .ok_or(Error::LoanComputationOverflow)?
                )
                .ok_or(Error::LoanComputationOverflow)?;

            let liquidation_price = loan_with_threshold
                .checked_div(collateral)
                .ok_or(Error::LoanComputationOverflow)?;

            self.loans.push(Loan {
                account,
                collateral,
                loan_amount,
                paid_amount: 0,
                balance: loan_amount,
                liquidation_price,
            });

            // Success
            self.env().emit_event(BankingEvent {
                operator: caller,
                status: BankTransactionStatus::EmitSuccess(Success::LoanApplicationSuccess),
            });

            Ok(())
        }

        /// Pay loan
        #[ink(message)]
        pub fn loan_payment(&mut self,
            account: AccountId,
            amount: u128) -> Result<(), Error> {

            // Loan payment can only be called by the manager after accepting USDT transfer
            let caller = self.env().caller();
            if self.env().caller() != self.manager {
                self.env().emit_event(BankingEvent {
                    operator: caller,
                    status: BankTransactionStatus::EmitError(Error::BadOrigin),
                });
                return Ok(());
            } 

            // Check if the bank is open
            if self.status != 0 {
                self.env().emit_event(BankingEvent {
                    operator: caller,
                    status: BankTransactionStatus::EmitError(Error::BankIsClose),
                });
                return Ok(());
            } 

            // Search for the loan
            let loan_index = match self.loans.iter().position(|l| l.account == account) {
                Some(i) => i,
                None => {
                    self.env().emit_event(BankingEvent {
                        operator: caller,
                        status: BankTransactionStatus::EmitError(Error::LoanNotFound),
                    });
                    return Ok(());
                }
            };          

            // If the amount is greater than or equal to the balance then we delete the loan (fully paid)
            if amount >= self.loans[loan_index].balance {
                // Remove the loan
                self.loans.remove(loan_index);

                // Find the ledger and add back the collateral to the account balance
                let l = self.loans[loan_index].clone();
                match self.ledgers.iter_mut().find(|l| l.account == account) {
                    Some(ledger) => {
                        ledger.balance = ledger.balance
                            .checked_add(l.collateral)
                            .ok_or(Error::LoanComputationOverflow)?;
                    },
                    None => {
                        self.env().emit_event(BankingEvent {
                            operator: caller,
                            status: BankTransactionStatus::EmitError(Error::AccountNotFound),
                        });
                        return Ok(());
                    }
                }

                self.env().emit_event(BankingEvent {
                    operator: caller,
                    status: BankTransactionStatus::EmitSuccess(Success::LoanFullyPaidSuccess),
                });
                return Ok(());
            }

            // Update the paid amount and balance
            let loan = &mut self.loans[loan_index];

            loan.paid_amount = loan.paid_amount
                .checked_add(amount)
                .ok_or(Error::LoanComputationOverflow)?;

            // Recompute balance
            loan.balance = loan.loan_amount
                .checked_sub(loan.paid_amount)
                .ok_or(Error::LoanComputationOverflow)?;
            

            self.env().emit_event(BankingEvent {
                operator: caller,
                status: BankTransactionStatus::EmitSuccess(Success::LoanPaymentSuccess),
            });

            Ok(())
        }
        
        /// Liquidate loan
        #[ink(message)]
        pub fn loan_liquidation(&mut self,
            price: u128) -> Result<(), Error> {

            // Loan payment can only be called by the manager based on the price oracle
            let caller = self.env().caller();
            if self.env().caller() != self.manager {
                self.env().emit_event(BankingEvent {
                    operator: caller,
                    status: BankTransactionStatus::EmitError(Error::BadOrigin),
                });
                return Ok(());
            } 

            // Check if the bank is open
            if self.status != 0 {
                self.env().emit_event(BankingEvent {
                    operator: caller,
                    status: BankTransactionStatus::EmitError(Error::BankIsClose),
                });
                return Ok(());
            } 

            // Loop through the loans and check if the liquidity price is higher than the price
            // liquidate the loan by removing it.
            let liquidation_indices: Vec<usize> = self.loans
                .iter()
                .enumerate()
                .filter(|(_, l)| l.liquidation_price >= price)
                .map(|(i, _)| i)
                .collect();

            // Remove in reverse order to preserve indices during removal
            for i in liquidation_indices.iter().rev() {
                self.loans.remove(*i);
            }

            self.env().emit_event(BankingEvent {
                operator: caller,
                status: BankTransactionStatus::EmitSuccess(Success::LoanLiquidationSuccess),
            });

            Ok(())
        }

        /// Get balance of an account
        #[ink(message)]
        pub fn get_balance(&self,
            account: AccountId) ->  Option<Ledger> {

            for ledger in self.ledgers.iter() {
                if ledger.account == account {
                    return Some(ledger.clone()); 
                }
            }

            None
        }

    }

    /// Unit tests
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            let Bank = Bank::default();
        }
    }


    /// This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
    ///
    /// When running these you need to make sure that you:
    /// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    /// - Are running a Substrate node which contains `pallet-contracts` in the background
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// A helper function used for calling contract messages.
        use ink_e2e::build_message;

        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// We test that we can upload and instantiate the contract using its default constructor.
        #[ink_e2e::test]
        async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = BankRef::default();

            // When
            let contract_account_id = client
                .instantiate("bank", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Then
            let get = build_message::<BankRef>(contract_account_id.clone())
                .call(|bank| bank.get());
            let get_result = client.call_dry_run(&ink_e2e::alice(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            Ok(())
        }

        /// We test that we can read and write a value from the on-chain contract contract.
        #[ink_e2e::test]
        async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = BankRef::new(false);
            let contract_account_id = client
                .instantiate("bank", &ink_e2e::bob(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let get = build_message::<BankRef>(contract_account_id.clone())
                .call(|bank| bank.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            // When
            let flip = build_message::<BankRef>(contract_account_id.clone())
                .call(|bank| bank.flip());
            let _flip_result = client
                .call(&ink_e2e::bob(), flip, 0, None)
                .await
                .expect("flip failed");

            // Then
            let get = build_message::<BankRef>(contract_account_id.clone())
                .call(|bank| bank.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), true));

            Ok(())
        }
    }
}
