multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub mod proxy;

pub const PRECISION: u64 = 1e18 as u64;

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, TypeAbi, Clone)]
pub struct TinderView<M: ManagedTypeApi> {
    pub state: u8,
    pub underlying_price_0: BigUint<M>,
    pub underlying_price_1: BigUint<M>,
}

#[multiversx_sc::module]
pub trait WrapperModule {
    #[proxy]
    fn proxy(&self, pool_address: ManagedAddress) -> proxy::Proxy<Self::Api>;

    #[view(getTinder)]
    fn get_tinder(&self, pool_address: ManagedAddress) -> TinderView<Self::Api> {
        let state: u8 = 1;
        let underlying_price_0: BigUint = BigUint::from(PRECISION);
        let underlying_price_1: BigUint = self.proxy(pool_address.clone()).get_share_to_assets_price().execute_on_dest_context();
        TinderView { state, underlying_price_0, underlying_price_1 }
    }
}