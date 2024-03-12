multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub mod proxy;

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, TypeAbi, Clone)]
pub struct AshswapV1View<M: ManagedTypeApi> {
    pub state: u8,
    pub reserves: ManagedVec<M, BigUint<M>>,
    pub underlying_prices: ManagedVec<M, BigUint<M>>,
    pub amp_factor: u64,
    pub swap_fee_percent: u64,
}

#[multiversx_sc::module]
pub trait WrapperModule {
    #[proxy]
    fn proxy(&self, pool_address: ManagedAddress) -> proxy::Proxy<Self::Api>;

    #[view(getAshswapV1)]
    fn get_ashswapv1(&self, pool_address: ManagedAddress) -> AshswapV1View<Self::Api> {
        let state: u8 = self.proxy(pool_address.clone()).get_state().execute_on_dest_context();
        let tokens: ManagedVec<TokenIdentifier> = self.proxy(pool_address.clone()).get_tokens().execute_on_dest_context();
        let amp_factor: u64 = self.proxy(pool_address.clone()).get_amp_factor().execute_on_dest_context();
        let swap_fee_percent: u64 = self.proxy(pool_address.clone()).swap_fee_percent().execute_on_dest_context();

        let precision = BigUint::from(1e18 as u64);
        let mut reserves = ManagedVec::new();
        let mut underlying_prices = ManagedVec::new();
        for token in tokens.into_iter() {
            reserves.push(self.proxy(pool_address.clone()).get_balances(&token).execute_on_dest_context());
            underlying_prices.push(self.proxy(pool_address.clone()).get_token_price(&token, &precision).execute_on_dest_context());
        }

        AshswapV1View { state, reserves, underlying_prices, amp_factor, swap_fee_percent }
    }
}
