#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, TypeAbi, Clone, ManagedVecItem)]
pub struct TokenAmount<M: ManagedTypeApi> {
    pub token: TokenIdentifier<M>,
    pub amount: BigUint<M>,
}

impl<M: ManagedTypeApi> TokenAmount<M> {
    pub fn new(token: TokenIdentifier<M>, amount: BigUint<M>) -> Self {
        TokenAmount { token, amount }
    }
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, TypeAbi, Clone, ManagedVecItem)]
pub struct AggregatorStep<M: ManagedTypeApi> {
    pub token_in: TokenIdentifier<M>,
    pub token_out: TokenIdentifier<M>,
    pub amount_in: BigUint<M>,
    pub pool_address: ManagedAddress<M>,
    pub function_name: ManagedBuffer<M>,
    pub arguments: ManagedVec<M, ManagedBuffer<M>>,
}
