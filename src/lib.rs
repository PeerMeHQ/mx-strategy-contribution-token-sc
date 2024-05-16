#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::contract]
pub trait StrategyContract {
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}
}
