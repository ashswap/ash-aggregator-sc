multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::proxy]
pub trait WrapperProxy {
    #[view(getState)]
    fn get_state(&self) -> u8;

    #[view(getExchangeRate)]
    fn get_exchange_rate(&self) -> BigUint;
}
