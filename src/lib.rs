#![no_std]

use errors::{ERR_APP_NOT_REGISTERED, ERR_APP_REGISTERED_ALREADY, ERR_PAYMENT_ZERO, ERR_TOKEN_INVALID, ERR_TOKEN_INVALID_ID};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub mod errors;
pub mod strategy_contribution_token_proxy;

const TOKEN_DECIMALS: usize = 18;

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub struct AppInfo<M: ManagedTypeApi> {
    pub contribution_token: TokenIdentifier<M>,
    pub receipt_token: TokenIdentifier<M>,
    pub receipt_token_supply: BigUint<M>,
}

#[multiversx_sc::contract]
pub trait StrategyContract {
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}

    /// Registers the app address and issues a new fungible token.
    #[payable("*")]
    #[endpoint(registerApp)]
    fn register_app_endpoint(&self, contribution_token: TokenIdentifier, receipt_token_name: ManagedBuffer, receipt_token_ticker: ManagedBuffer) {
        // TODO: remove later
        self.app_infos(&self.blockchain().get_caller()).clear();

        require!(contribution_token.is_valid_esdt_identifier(), ERR_TOKEN_INVALID_ID);

        let payment = self.call_value().egld_value().clone_value();
        require!(payment > 0, ERR_PAYMENT_ZERO);

        let app = self.blockchain().get_caller();
        require!(self.app_infos(&app).is_empty(), ERR_APP_REGISTERED_ALREADY);

        self.tx()
            .to(ESDTSystemSCAddress)
            .typed(ESDTSystemSCProxy)
            .issue_and_set_all_roles(payment, receipt_token_name, receipt_token_ticker, EsdtTokenType::Fungible, TOKEN_DECIMALS)
            .callback(self.callbacks().token_issue_callback(app, contribution_token))
            .async_call_and_exit();
    }

    /// Forwards payments to the app address and mints the token proportionally to the payment amount.
    /// Minted tokens are then transferred to the caller.
    #[payable("*")]
    #[endpoint(participate)]
    fn participate_endpoint(&self, app: ManagedAddress) {
        let caller = self.blockchain().get_caller();
        let app_info = self.get_app_info_or_fail(&app);

        let payment = self.call_value().single_esdt();
        require!(payment.amount > 0, ERR_PAYMENT_ZERO);
        require!(payment.token_identifier == app_info.contribution_token, ERR_TOKEN_INVALID);

        self.tx()
            .to(ToSelf)
            .typed(system_proxy::UserBuiltinProxy)
            .esdt_local_mint(&app_info.receipt_token, 0, &payment.amount)
            .sync_call();

        let minted_payment = EsdtTokenPayment::new(app_info.receipt_token, 0, payment.amount.clone());

        self.increase_app_receipt_token_supply(&app, payment.amount.clone());

        self.tx().to(&app).esdt(payment).transfer();
        self.tx().to(&caller).esdt(minted_payment).transfer();
    }

    #[payable("*")]
    #[callback]
    fn token_issue_callback(&self, app: ManagedAddress, contribution_token: TokenIdentifier, #[call_result] result: ManagedAsyncCallResult<TokenIdentifier>) {
        match result {
            ManagedAsyncCallResult::Ok(issued_token) => {
                self.app_infos(&app).set(AppInfo {
                    contribution_token,
                    receipt_token: issued_token,
                    receipt_token_supply: BigUint::zero(),
                });
            }
            ManagedAsyncCallResult::Err(_) => self.send_received_egld(&app),
        }
    }

    fn get_app_info_or_fail(&self, address: &ManagedAddress) -> AppInfo<Self::Api> {
        require!(!self.app_infos(address).is_empty(), ERR_APP_NOT_REGISTERED);

        self.app_infos(address).get()
    }

    fn increase_app_receipt_token_supply(&self, app: &ManagedAddress, amount: BigUint) {
        let mut app_info = self.get_app_info_or_fail(app);
        app_info.receipt_token_supply += amount;
        self.app_infos(app).set(app_info);
    }

    fn send_received_egld(&self, to: &ManagedAddress) {
        let egld_received = self.call_value().egld_value().clone_value();
        if egld_received > 0 {
            self.send().direct_egld(&to, &egld_received);
        }
    }

    #[storage_mapper("members")]
    fn members(&self, address: &ManagedAddress) -> MapMapper<ManagedAddress, BigUint>;

    #[storage_mapper("app_infos")]
    fn app_infos(&self, address: &ManagedAddress) -> SingleValueMapper<AppInfo<Self::Api>>;
}
