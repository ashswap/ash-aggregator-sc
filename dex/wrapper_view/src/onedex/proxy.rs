multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, TypeAbi, Clone, ManagedVecItem)]
pub struct OnedexPool<M: ManagedTypeApi> {
    pub pair_id: u32,
    pub state: u8,
    pub enabled: bool,
    pub owner: ManagedAddress<M>,
    pub first_token_id: TokenIdentifier<M>,
    pub second_token_id: TokenIdentifier<M>,
    pub lp_token_id: TokenIdentifier<M>,
    pub lp_token_decimal: u32,
    pub first_token_reserve: BigUint<M>,
    pub second_token_reserve: BigUint<M>,
    pub lp_token_supply: BigUint<M>,
    pub lp_token_roles_are_set: bool,
    pub total_fee_percentage: u64,
}

#[multiversx_sc::proxy]
pub trait WrapperProxy {
    #[view(viewPairs)]
    fn view_pairs(&self) -> MultiValueEncoded<OnedexPool<Self::Api>>;
}
