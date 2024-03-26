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
    pub token_id_0: TokenIdentifier<M>,
    pub token_id_1: TokenIdentifier<M>,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, TypeAbi, Clone, ManagedVecItem)]
pub struct OnedexViewRequest<M: ManagedTypeApi> {
    pub pool_address: ManagedAddress<M>,
    pub token_id_0: TokenIdentifier<M>,
    pub token_id_1: TokenIdentifier<M>,
}

#[multiversx_sc::module]
pub trait WrapperModule {
    #[proxy]
    fn proxy(&self, pool_address: ManagedAddress) -> proxy::Proxy<Self::Api>;

    #[view(getOnedex)]
    fn get_onedex(&self, request: MultiValueEncoded<OnedexViewRequest<Self::Api>>) -> MultiValueEncoded<OnedexView<Self::Api>> {
        let mut result = MultiValueEncoded::new();
        let mut pairs: Option<MultiValueEncoded<OnedexPool<Self::Api>>> = Option::None;
        let mut total_fee: Option<BigUint> = Option::None;
        for req in request.into_iter() {
            let pool_address = req.pool_address.clone();
            if pairs.is_none() {
                pairs = Option::Some(self.proxy(req.pool_address.clone()).view_pairs().execute_on_dest_context());
                total_fee = Option::Some(self.proxy(pool_address.clone()).get_total_fee_percent().execute_on_dest_context());
            }
            let token_id_0 = req.token_id_0.clone();
            let token_id_1 = req.token_id_1.clone();
            for pair in pairs.clone().unwrap().into_iter() {
                if pair.token_id_0 == token_id_0 && pair.token_id_1 == token_id_1 {
                    let view = OnedexView {
                        status: pair.status,
                        total_fee: total_fee.clone().unwrap(),
                        reserve_0: pair.reserve_0,
                        reserve_1: pair.reserve_1,
                        address: pool_address.clone(),
                        token_id_0: token_id_0.clone(),
                        token_id_1: token_id_1.clone(),
                    };
                    result.push(view);
                }
            }
        }
        result
    }
}
