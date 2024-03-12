multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub mod proxy;

use self::proxy::*;

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, TypeAbi, Clone)]
pub struct JexExchangeView<M: ManagedTypeApi> {
    pub paused: bool,
    pub reserve_0: BigUint<M>,
    pub reserve_1: BigUint<M>,
    pub lp_fees: u32,
    pub platform_fees: u32,
}

#[multiversx_sc::module]
pub trait WrapperModule {
    #[proxy]
    fn proxy(&self, pool_address: ManagedAddress) -> proxy::Proxy<Self::Api>;

    #[view(getJexExchange)]
    fn get_jexexchange(&self, pool_address: ManagedAddress) -> JexExchangeView<Self::Api> {
        let pair: JexExchangePool<Self::Api> = self.proxy(pool_address.clone()).get_status().execute_on_dest_context();
        JexExchangeView {
            paused: pair.paused,
            reserve_0: pair.first_token_reserve,
            reserve_1: pair.second_token_reserve,
            lp_fees: pair.lp_fees,
            platform_fees: pair.platform_fees,
        }
    }
}
