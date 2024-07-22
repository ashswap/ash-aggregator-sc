multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub mod proxy;

use self::proxy::*;

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, TypeAbi, Clone)]
pub struct OnedexView<M: ManagedTypeApi> {
    pub status: u8,
    pub total_fee: BigUint<M>,
    pub reserve_0: BigUint<M>,
    pub reserve_1: BigUint<M>,
    pub address: ManagedAddress<M>,
    pub token_0_id: TokenIdentifier<M>,
    pub token_1_id: TokenIdentifier<M>,
}

#[multiversx_sc::module]
pub trait WrapperModule {
    #[proxy]
    fn proxy(&self, pool_address: ManagedAddress) -> proxy::Proxy<Self::Api>;

    #[view(getOnedex)]
    fn get_onedex(&self, pool_address: ManagedAddress) -> MultiValueEncoded<OnedexView<Self::Api>> {
        let pairs: MultiValueEncoded<OnedexPool<Self::Api>> = self.proxy(pool_address.clone()).view_pairs().execute_on_dest_context();

        let mut result = MultiValueEncoded::new();
        for pair in pairs.into_iter() {
            result.push(OnedexView {
                status: pair.state,
                total_fee: BigUint::from(pair.total_fee_percentage),
                reserve_0: pair.first_token_reserve,
                reserve_1: pair.second_token_reserve,
                address: pool_address.clone(),
                token_0_id: pair.first_token_id,
                token_1_id: pair.second_token_id,
            });
        }
        result
    }
}
