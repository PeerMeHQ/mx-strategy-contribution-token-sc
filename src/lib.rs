#![no_std]

use errors::{ERR_APP_NOT_REGISTERED, ERR_APP_REGISTERED_ALREADY, ERR_PAYMENT_ZERO, ERR_TOKEN_INVALID, ERR_TOKEN_INVALID_ID, ERR_TOKEN_NOT_ISSUED};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub mod errors;
pub mod strategy_contribution_token_proxy;

const TOKEN_DECIMALS: usize = 18;

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub struct AppInfo<M: ManagedTypeApi> {
    pub contribution_token: TokenIdentifier<M>,
    pub receipt_token: Option<TokenIdentifier<M>>,
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
        require!(contribution_token.is_valid_esdt_identifier(), ERR_TOKEN_INVALID_ID);

        let payment = self.call_value().egld_value().clone_value();
        require!(payment > 0, ERR_PAYMENT_ZERO);

        let app = self.blockchain().get_caller();
        require!(self.app_infos(&app).is_empty(), ERR_APP_REGISTERED_ALREADY);

        self.app_infos(&app).set(AppInfo {
            contribution_token,
            receipt_token: Option::None,
            receipt_token_supply: BigUint::zero(),
        });

        self.tx()
            .to(ESDTSystemSCAddress)
            .typed(ESDTSystemSCProxy)
            .issue_and_set_all_roles(payment, receipt_token_name, receipt_token_ticker, EsdtTokenType::Fungible, TOKEN_DECIMALS)
            .callback(self.callbacks().token_issue_callback(&app))
            .async_call_and_exit();
    }

    /// Forwards payments to the app address and mints the token proportionally to the payment amount.
    /// Minted tokens are then transferred to the caller.
    #[payable("*")]
    #[endpoint(participate)]
    fn participate_endpoint(&self, app: ManagedAddress) {
        let caller = self.blockchain().get_caller();
        let app_info = self.get_app_info_or_fail(&app);
        require!(app_info.receipt_token.is_some(), ERR_TOKEN_NOT_ISSUED);

        let payment = self.call_value().single_esdt();
        require!(payment.amount > 0, ERR_PAYMENT_ZERO);
        require!(payment.token_identifier == app_info.contribution_token, ERR_TOKEN_INVALID);

        let token = app_info.receipt_token.unwrap();

        self.tx()
            .to(ToSelf)
            .typed(system_proxy::UserBuiltinProxy)
            .esdt_local_mint(&token, 0, &payment.amount)
            .sync_call();

        let minted_payment = EsdtTokenPayment::new(token, 0, payment.amount.clone());

        self.tx().to(&app).esdt(payment).transfer();
        self.tx().to(&caller).esdt(minted_payment).transfer();
    }

    #[payable("*")]
    #[callback]
    fn token_issue_callback(&self, app: &ManagedAddress, #[call_result] result: ManagedAsyncCallResult<TokenIdentifier>) {
        match result {
            ManagedAsyncCallResult::Ok(token_id) => {
                let mut app_info = self.get_app_info_or_fail(&app);
                app_info.receipt_token = Option::Some(token_id);
                self.app_infos(&app).set(app_info);
            }
            ManagedAsyncCallResult::Err(_) => self.send_received_egld(&app),
        }
    }

    fn get_app_info_or_fail(&self, address: &ManagedAddress) -> AppInfo<Self::Api> {
        require!(!self.app_infos(address).is_empty(), ERR_APP_NOT_REGISTERED);

        self.app_infos(address).get()
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
