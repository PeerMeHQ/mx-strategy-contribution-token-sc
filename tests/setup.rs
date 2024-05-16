multiversx_sc::imports!();

use multiversx_sc_scenario::testing_framework::*;
use multiversx_sc_scenario::*;
use strategy_token::*;

pub const WASM_PATH: &'static str = "output/strategy.wasm";

#[allow(dead_code)]
pub struct Setup<ObjBuilder>
where
    ObjBuilder: 'static + Copy + Fn() -> strategy_token::ContractObj<DebugApi>,
{
    pub blockchain: BlockchainStateWrapper,
    pub owner_address: Address,
    pub user_address: Address,
    pub resolver_address: Address,
    pub contract: ContractObjWrapper<strategy_token::ContractObj<DebugApi>, ObjBuilder>,
}

impl<ObjBuilder> Setup<ObjBuilder>
where
    ObjBuilder: 'static + Copy + Fn() -> strategy_token::ContractObj<DebugApi>,
{
    pub fn new(builder: ObjBuilder) -> Self {
        let rust_zero = rust_biguint!(0u64);
        let mut blockchain = BlockchainStateWrapper::new();
        let owner_address = blockchain.create_user_account(&rust_zero);
        let user_address = blockchain.create_user_account(&rust_zero);
        let resolver_address = blockchain.create_user_account(&rust_zero);
        let contract = blockchain.create_sc_account(&rust_zero, Some(&owner_address), builder, WASM_PATH);

        blockchain
            .execute_tx(&owner_address, &contract, &rust_zero, |sc| {
                sc.init();
            })
            .assert_ok();

        Setup {
            blockchain,
            owner_address,
            user_address,
            resolver_address,
            contract,
        }
    }
}

#[test]
fn it_initializes_the_contract() {
    let mut setup = Setup::new(strategy - token::contract_obj);

    setup.blockchain.execute_query(&setup.contract, |_| {}).assert_ok();
}
