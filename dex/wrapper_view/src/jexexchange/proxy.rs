multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, TypeAbi, Clone, ManagedVecItem)]
pub struct JexExchangePool<M: ManagedTypeApi> {
    pub paused: u8,
    pub first_token_identifier: TokenIdentifier<M>,
    pub first_token_reserve: BigUint<M>,
    pub second_token_identifier: TokenIdentifier<M>,
    pub second_token_reserve: BigUint<M>,
    pub lp_token_identifier: TokenIdentifier<M>,
    pub lp_token_supply: BigUint<M>,
    pub owner: ManagedAddress<M>,
    pub lp_fees: u32,
    pub platform_fees: u32,
}

#[multiversx_sc::proxy]
pub trait WrapperProxy {
    #[view(getStatus)]
    fn get_status(&self) -> ManagedVec<JexExchangePool<Self::Api>>;
}
