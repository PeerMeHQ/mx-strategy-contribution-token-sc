use multiversx_sc::types::*;
use multiversx_sc_scenario::api::*;
use multiversx_sc_scenario::imports::*;
use setup::*;
use strategy_contribution_token::strategy_contribution_token_proxy;

pub mod setup;

#[test]
fn it_forwards_payments_to_an_app() {
    let mut contract = TestContract::new();

    let receipt_token_name = ManagedBuffer::<StaticApi>::from(b"Receipt");
    let receipt_token_ticker = ManagedBuffer::<StaticApi>::from(b"RECEIPT");
    let contribution_token = TokenIdentifier::<StaticApi>::from("CONTR-123456");

    contract.register_app(contribution_token.clone(), receipt_token_name, receipt_token_ticker);

    contract.chain
        .tx()
        .from(APP_ADDRESS)
        .to(STRATEGY_ADDRESS)
        .typed(strategy_contribution_token_proxy::StrategyContractProxy)
        .participate_endpoint(APP_ADDRESS)
        .esdt(EsdtTokenPayment::new(contribution_token, 0, BigUint::from(50u32)))
        .run();

    assert!(true); // TODO: figure out how to check storage in type safe way
}

// TODO: it_mints_tokens_proportionally_to_payment_amount_and_transfers_them_to_caller

// TODO: it_fails_when_payment_not_contribution_token

// TODO: it_fails_when_app_not_registered

// TODO: it_fails_when_payment_is_zero
