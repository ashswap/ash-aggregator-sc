multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub mod proxy;

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, TypeAbi, Clone)]
pub struct XExchangeView<M: ManagedTypeApi> {
    pub state: u8,
    pub total_fee: u64,
    pub reserve_0: BigUint<M>,
    pub reserve_1: BigUint<M>,
}

#[multiversx_sc::module]
pub trait WrapperModule {
    #[proxy]
    fn proxy(&self, pool_address: ManagedAddress) -> proxy::Proxy<Self::Api>;

    #[view(getXExchange)]
    fn get_xexchange(&self, pool_address: ManagedAddress, token_id_0: TokenIdentifier, token_id_1: TokenIdentifier) -> XExchangeView<Self::Api> {
        let state: u8 = self.proxy(pool_address.clone()).get_state().execute_on_dest_context();
        let total_fee: u64 = self.proxy(pool_address.clone()).get_total_fee_percent().execute_on_dest_context();
        let reserve_0: BigUint = self.proxy(pool_address.clone()).get_reserve(token_id_0).execute_on_dest_context();
        let reserve_1: BigUint = self.proxy(pool_address.clone()).get_reserve(token_id_1).execute_on_dest_context();
        XExchangeView { state, total_fee, reserve_0, reserve_1 }
    }
}
