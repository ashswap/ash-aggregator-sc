multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, TypeAbi, Clone, ManagedVecItem)]
pub struct OnedexPool<M: ManagedTypeApi> {
    pub id: u32,
    pub status: u8,
    pub unknown_0: u8,
    pub pool_owner: ManagedAddress<M>,
    pub token_id_0: TokenIdentifier<M>,
    pub token_id_1: TokenIdentifier<M>,
    pub lp_token_id: TokenIdentifier<M>,
    pub lp_decimal: u32,
    pub reserve_0: BigUint<M>,
    pub reserve_1: BigUint<M>,
    pub total_lp: BigUint<M>,
    pub unknown_1: u8,
}

#[multiversx_sc::proxy]
pub trait WrapperProxy {
    #[view(getTotalFeePercent)]
    fn get_total_fee_percent(&self) -> BigUint;

    #[view(viewPairs)]
    fn view_pairs(&self) -> ManagedVec<OnedexPool<Self::Api>>;
}
