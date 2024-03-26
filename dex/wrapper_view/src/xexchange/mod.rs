multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub mod proxy;

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, TypeAbi, Clone, ManagedVecItem)]
pub struct XExchangeView<M: ManagedTypeApi> {
    pub address: ManagedAddress<M>,
    pub state: u8,
    pub total_fee: u64,
    pub token_0_id: TokenIdentifier<M>,
    pub token_1_id: TokenIdentifier<M>,
    pub reserve_0: BigUint<M>,
    pub reserve_1: BigUint<M>,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, TypeAbi, Clone, ManagedVecItem)]
pub struct XExchangeViewRequest<M: ManagedTypeApi> {
    pub pool_address: ManagedAddress<M>,
    pub token_id_0: TokenIdentifier<M>,
    pub token_id_1: TokenIdentifier<M>,
}

#[multiversx_sc::module]
pub trait WrapperModule {
    #[proxy]
    fn proxy(&self, pool_address: ManagedAddress) -> proxy::Proxy<Self::Api>;

    #[view(getXExchange)]
    fn get_xexchange(&self, request: MultiValueEncoded<XExchangeViewRequest<Self::Api>>) -> MultiValueEncoded<XExchangeView<Self::Api>> {
        let mut result = MultiValueEncoded::new();
        for req in request.into_iter() {
            let pool_address = req.pool_address.clone();
            let token_id_0 = req.token_id_0.clone();
            let token_id_1 = req.token_id_1.clone();
            let state: u8 = self.proxy(pool_address.clone()).get_state().execute_on_dest_context();
            let total_fee: u64 = self.proxy(pool_address.clone()).get_total_fee_percent().execute_on_dest_context();
            let reserve_0: BigUint = self.proxy(pool_address.clone()).get_reserve(token_id_0.clone()).execute_on_dest_context();
            let reserve_1: BigUint = self.proxy(pool_address.clone()).get_reserve(token_id_1.clone()).execute_on_dest_context();
            let view = XExchangeView {
                address: pool_address,
                state, 
                total_fee,
                token_0_id: token_id_0,
                token_1_id: token_id_1,
                reserve_0, 
                reserve_1 
            };
            result.push(view);
        }
        result
    }
}
