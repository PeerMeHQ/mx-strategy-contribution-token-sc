use multiversx_sc_scenario::imports::*;

pub const OWNER_ADDRESS: TestAddress = TestAddress::new("owner");
pub const STRATEGY_ADDRESS: TestSCAddress = TestSCAddress::new("strategy");
pub const CODE_PATH: MxscPath = MxscPath::new("output/adder.mxsc.json");

pub fn blockchain() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();
    blockchain.register_contract(CODE_PATH, strategy_token::ContractBuilder);

    blockchain
}

pub struct TestContract {
    pub chain: ScenarioWorld,
}

impl TestContract {
    pub fn new() -> Self {
        let chain = blockchain();

        Self { chain }
    }
}
