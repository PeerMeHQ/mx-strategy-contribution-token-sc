#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

const TOKEN_DECIMALS: usize = 18;

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub struct EntityInfo<M: ManagedTypeApi> {
    pub accepted_token: TokenIdentifier<M>,
    pub token: Option<TokenIdentifier<M>>,
    pub token_supply: BigUint<M>,
}

#[multiversx_sc::contract]
pub trait StrategyContract {
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}

    #[payable("*")]
    #[endpoint(register)]
    fn register_endpoint(&self, token_name: ManagedBuffer, token_ticker: ManagedBuffer, accepted_token: TokenIdentifier) {
        require!(accepted_token.is_valid_esdt_identifier(), "invalid token id");

        let payment = self.call_value().egld_value().clone_value();
        require!(payment > 0, "must be more than 0");

        let entity = self.blockchain().get_caller();
        require!(self.blockchain().is_smart_contract(&entity), "not a contract");
        require!(self.entity_infos(&entity).is_empty(), "already registered");

        self.entity_infos(&entity).set(EntityInfo {
            accepted_token,
            token: Option::None,
            token_supply: BigUint::zero(),
        });

        self.tx()
            .to(ESDTSystemSCAddress)
            .typed(ESDTSystemSCProxy)
            .issue_and_set_all_roles(payment, token_name, token_ticker, EsdtTokenType::Fungible, TOKEN_DECIMALS)
            .callback(self.callbacks().token_issue_callback(&entity))
            .async_call_and_exit();
    }

    #[payable("*")]
    #[endpoint(participate)]
    fn participate_endpoint(&self, entity: ManagedAddress) {
        let caller = self.blockchain().get_caller();
        let entity_info = self.get_entity_info_or_fail(&entity);
        require!(entity_info.token.is_some(), "token not issued");

        let payment = self.call_value().single_esdt();
        require!(payment.amount > 0, "must be more than 0");
        require!(payment.token_identifier == entity_info.accepted_token, "invalid token");

        let token = entity_info.token.unwrap();

        self.tx()
            .to(ToSelf)
            .typed(system_proxy::UserBuiltinProxy)
            .esdt_local_mint(&token, 0, &payment.amount)
            .sync_call();

        let minted_payment = EsdtTokenPayment::new(token, 0, payment.amount.clone());

        self.tx().to(&entity).esdt(payment).transfer();
        self.tx().to(&caller).esdt(minted_payment).transfer();
    }

    #[payable("*")]
    #[callback]
    fn token_issue_callback(&self, entity: &ManagedAddress, #[call_result] result: ManagedAsyncCallResult<TokenIdentifier>) {
        match result {
            ManagedAsyncCallResult::Ok(token_id) => {
                let mut entity_info = self.get_entity_info_or_fail(&entity);
                entity_info.token = Option::Some(token_id);
                self.entity_infos(&entity).set(entity_info);
            }
            ManagedAsyncCallResult::Err(_) => self.send_received_egld(&entity),
        }
    }

    fn get_entity_info_or_fail(&self, address: &ManagedAddress) -> EntityInfo<Self::Api> {
        require!(!self.entity_infos(address).is_empty(), "entity not found");

        self.entity_infos(address).get()
    }

    fn send_received_egld(&self, to: &ManagedAddress) {
        let egld_received = self.call_value().egld_value().clone_value();
        if egld_received > 0 {
            self.send().direct_egld(&to, &egld_received);
        }
    }

    #[storage_mapper("members")]
    fn members(&self, address: &ManagedAddress) -> MapMapper<ManagedAddress, BigUint>;

    #[storage_mapper("entity_infos")]
    fn entity_infos(&self, address: &ManagedAddress) -> SingleValueMapper<EntityInfo<Self::Api>>;
}
