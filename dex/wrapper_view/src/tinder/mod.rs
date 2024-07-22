multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub mod proxy;

pub const PRECISION: u64 = 1e18 as u64;

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, TypeAbi, Clone)]
pub struct TinderView<M: ManagedTypeApi> {
    pub state: u8,
    pub underlying_price_0: BigUint<M>,
    pub underlying_price_1: BigUint<M>,
    pub address: ManagedAddress<M>,
    pub token_id_0: TokenIdentifier<M>,
    pub token_id_1: TokenIdentifier<M>,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, TypeAbi, Clone, ManagedVecItem)]
pub struct TinderViewRequest<M: ManagedTypeApi> {
    pub pool_address: ManagedAddress<M>,
    pub token_id_0: TokenIdentifier<M>,
    pub token_id_1: TokenIdentifier<M>,
}

#[multiversx_sc::module]
pub trait WrapperModule {
    #[proxy]
    fn proxy(&self, pool_address: ManagedAddress) -> proxy::Proxy<Self::Api>;

    #[view(getTinder)]
    fn get_tinder(&self, request: MultiValueEncoded<TinderViewRequest<Self::Api>>) -> MultiValueEncoded<TinderView<Self::Api>> {
        let mut result = MultiValueEncoded::new();
        for req in request.into_iter() {
            let pool_address = req.pool_address.clone();
            let token_id_0 = req.token_id_0.clone();
            let token_id_1 = req.token_id_1.clone();
            let state: u8 = 1;
            let underlying_price_0: BigUint = BigUint::from(PRECISION);
            let underlying_price_1: BigUint = self.proxy(pool_address.clone()).get_share_to_assets_price().execute_on_dest_context();
            let view = TinderView {
                state,
                underlying_price_0,
                underlying_price_1,
                address: pool_address,
                token_id_0,
                token_id_1,
            };
            result.push(view);
        }
        result
    }
}
