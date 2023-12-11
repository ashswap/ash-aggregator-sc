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
}

impl<M: ManagedTypeApi> OnedexView<M>  {
    pub fn empty() -> Self {
        Self {
            status: 0,
            total_fee: BigUint::zero(),
            reserve_0: BigUint::zero(),
            reserve_1: BigUint::zero(),
        }
    }
}

#[multiversx_sc::module]
pub trait WrapperModule {
    #[proxy]
    fn proxy(&self, pool_address: ManagedAddress) -> proxy::Proxy<Self::Api>;

    #[view(getOnedex)]
    fn get_onedex(&self, pool_address: ManagedAddress, token_id_0: TokenIdentifier, token_id_1: TokenIdentifier) -> OnedexView<Self::Api> {
        let total_fee: BigUint = self.proxy(pool_address.clone()).get_total_fee_percent().execute_on_dest_context();
        let pairs: ManagedVec<OnedexPool<Self::Api>> = self.proxy(pool_address.clone()).view_pairs().execute_on_dest_context();

        for pair in pairs.into_iter() {
            if pair.token_id_0 == token_id_0 && pair.token_id_1 == token_id_1 {
                return OnedexView {
                    status: pair.status,
                    total_fee,
                    reserve_0: pair.reserve_0,
                    reserve_1: pair.reserve_1,
                };
            }
        }
        OnedexView::empty()
    }
}
