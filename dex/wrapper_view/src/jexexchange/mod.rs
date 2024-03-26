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
    pub address: ManagedAddress<M>,
    pub token_id_0: TokenIdentifier<M>,
    pub token_id_1: TokenIdentifier<M>,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, TypeAbi, Clone, ManagedVecItem)]
pub struct JexExchangeViewRequest<M: ManagedTypeApi> {
    pub pool_address: ManagedAddress<M>,
    pub token_id_0: TokenIdentifier<M>,
    pub token_id_1: TokenIdentifier<M>,
}

#[multiversx_sc::module]
pub trait WrapperModule {
    #[proxy]
    fn proxy(&self, pool_address: ManagedAddress) -> proxy::Proxy<Self::Api>;

    #[view(getJexExchange)]
    fn get_jexexchange(&self, request: MultiValueEncoded<JexExchangeViewRequest<Self::Api>>) -> MultiValueEncoded<JexExchangeView<Self::Api>> {
        let mut result = MultiValueEncoded::new();
        for req in request.into_iter() {
            let pool_address = req.pool_address.clone();
            let token_id_0 = req.token_id_0.clone();
            let token_id_1 = req.token_id_1.clone();
            let pair: JexExchangePool<Self::Api> = self.proxy(pool_address.clone()).get_status().execute_on_dest_context();
            let view = JexExchangeView {
                paused: pair.paused,
                reserve_0: pair.first_token_reserve,
                reserve_1: pair.second_token_reserve,
                lp_fees: pair.lp_fees,
                platform_fees: pair.platform_fees,
                address: pool_address,
                token_id_0,
                token_id_1,
            };
            result.push(view);
        }
        result
    }
}
