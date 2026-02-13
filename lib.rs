#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod bank {

    use ink::prelude::vec::Vec;

    /// Error Messages
    #[derive(scale::Encode, scale::Decode, Debug, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Bad origin error, e.g., wrong caller
        BadOrigin,
        /// There is already an existing account
        AccountAlreadyExist,
        /// Account not found
        AccountNotFound,
        /// Account Balance Insufficient
        AccountBalanceInsufficient,
    }

    /// Success Messages
    #[derive(scale::Encode, scale::Decode, Debug, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Success {
        /// Bank setup successful
        BankSetupSuccess,
        /// Account deposit successful
        AccountDepositSuccess,
        /// Account withdrawal successful
        AccountWithdrawalSuccess,
        /// Account debit success
        AccountDebitSuccess,
        /// Account credit success
        AccountCreditSuccess,        
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
        /// Status (0-Frozen, 1-Liquid)
        pub status: u8,
    }        

    /// Bank storage
    #[ink(storage)]
    pub struct Bank {
        /// Bank asset
        pub asset_id: u128,
        /// Bank owner
        pub owner: AccountId,
        /// Bank manager
        pub manager: AccountId,
        /// Maximum accounts the bank ledger can handle
        pub maximum_accounts: u16,
        /// Bank ledgers
        pub ledgers: Vec<Ledger>,
        /// Status (0-Open, 1-Close)
        pub status: u8,
    }

    impl Bank {

        /// Create new bank
        #[ink(constructor)]
        pub fn new(asset_id: u128, 
            maximum_accounts: u16) -> Self {

            let caller: ink::primitives::AccountId = Self::env().caller();

            Self { 
                asset_id: asset_id, 
                owner: caller,
                manager: caller,
                maximum_accounts: maximum_accounts,
                ledgers: Vec::new(),
                status: 0u8,
            }
        }

        /// Default setup
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(0u128, 0u16)
        }

        /// Setup bank
        #[ink(message)]
        pub fn setup(&mut self,
            asset_id: u128,
            manager: AccountId,
            maximum_accounts: u16) -> Result<(), Error> {
            
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
            self.manager = manager;
            self.maximum_accounts = maximum_accounts;
            self.ledgers =  Vec::new();

            self.env().emit_event(BankingEvent {
                operator: caller,
                status: BankTransactionStatus::EmitSuccess(Success::BankSetupSuccess),
            });

            Ok(())
        }

        /// Get the bank information
        #[ink(message)]
        pub fn get(&self) -> (u128, AccountId, AccountId, u16, u8) {
            (
                self.asset_id,
                self.owner,
                self.manager,
                self.maximum_accounts,
                self.status,
            )
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
