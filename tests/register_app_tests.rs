use multiversx_sc::types::*;
use multiversx_sc_scenario::api::*;
use setup::*;

pub mod setup;

#[test]
fn it_registers_an_app() {
    let mut contract = TestContract::new();

    let receipt_token_name = ManagedBuffer::<StaticApi>::from(b"Receipt");
    let receipt_token_ticker = ManagedBuffer::<StaticApi>::from(b"RECEIPT");
    let contribution_token = TokenIdentifier::<StaticApi>::from("CONTR-123456");

    contract.register_app(contribution_token, receipt_token_name, receipt_token_ticker);

    assert!(true); // TODO: figure out how to check storage in type safe way
}

// TODO: it_fails_when_app_registered_already
