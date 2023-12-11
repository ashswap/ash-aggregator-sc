multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub mod proxy;

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, TypeAbi, Clone)]
pub struct AshswapV2View<M: ManagedTypeApi> {
    pub total_supply: BigUint<M>,
    pub reserves: ManagedVec<M, BigUint<M>>,
    pub real_reserves: ManagedVec<M, BigUint<M>>,
    pub amp_factor: BigUint<M>,
    pub gamma: BigUint<M>,
    pub virtual_price: BigUint<M>,
    pub price_oracle: BigUint<M>,
    pub price_scale: BigUint<M>,
    pub future_a_gamma_time: u64,
    pub initial_a_gamma_time: u64,
    pub d: BigUint<M>,
    pub mid_fee: BigUint<M>,
    pub out_fee: BigUint<M>,
    pub fee_gamma: BigUint<M>,
    pub state: u8,
    pub allowed_extra_profit: BigUint<M>,
    pub adjustment_step: BigUint<M>,
    pub admin_fee: BigUint<M>,
    pub last_prices: BigUint<M>,
    pub last_price_ts: u64,
    pub ma_half_time: u64,
    pub xcp_profit: BigUint<M>,
    pub xcp_profit_a: BigUint<M>,
    pub is_not_adjusted: bool,
}

#[multiversx_sc::module]
pub trait WrapperModule {
    #[proxy]
    fn proxy(&self, pool_address: ManagedAddress) -> proxy::Proxy<Self::Api>;

    #[view(getAshswapV2)]
    fn get_xexchange(&self, pool_address: ManagedAddress, token_0: TokenIdentifier, token_1: TokenIdentifier) -> AshswapV2View<Self::Api> {
        let state: u8 = self.proxy(pool_address.clone()).get_state().execute_on_dest_context();
        let total_supply: BigUint<Self::Api> = self.proxy(pool_address.clone()).get_lp_token_supply().execute_on_dest_context();
        let reserves: ManagedVec<Self::Api, BigUint<Self::Api>> = self.proxy(pool_address.clone()).get_balances().execute_on_dest_context();
        let amp_factor: BigUint<Self::Api> = self.proxy(pool_address.clone()).get_a().execute_on_dest_context();
        let gamma: BigUint<Self::Api> = self.proxy(pool_address.clone()).get_gamma().execute_on_dest_context();
        let virtual_price: BigUint<Self::Api> = self.proxy(pool_address.clone()).get_virtual_price().execute_on_dest_context();
        let price_oracle: BigUint<Self::Api> = self.proxy(pool_address.clone()).get_price_oracle().execute_on_dest_context();
        let price_scale: BigUint<Self::Api> = self.proxy(pool_address.clone()).get_price_scale().execute_on_dest_context();
        let future_a_gamma_time: u64 = self.proxy(pool_address.clone()).get_future_a_gamma_time().execute_on_dest_context();
        let initial_a_gamma_time: u64 = self.proxy(pool_address.clone()).get_initial_a_gamma_time().execute_on_dest_context();
        let d: BigUint<Self::Api> = self.proxy(pool_address.clone()).get_d().execute_on_dest_context();
        let mid_fee: BigUint<Self::Api> = self.proxy(pool_address.clone()).get_mid_fee().execute_on_dest_context();
        let out_fee: BigUint<Self::Api> = self.proxy(pool_address.clone()).get_out_fee().execute_on_dest_context();
        let fee_gamma: BigUint<Self::Api> = self.proxy(pool_address.clone()).get_fee_gamma().execute_on_dest_context();
        let allowed_extra_profit: BigUint<Self::Api> = self.proxy(pool_address.clone()).get_allowed_extra_profit().execute_on_dest_context();
        let adjustment_step: BigUint<Self::Api> = self.proxy(pool_address.clone()).get_adjustment_step().execute_on_dest_context();
        let admin_fee: BigUint<Self::Api> = self.proxy(pool_address.clone()).get_admin_fee().execute_on_dest_context();
        let last_prices: BigUint<Self::Api> = self.proxy(pool_address.clone()).get_last_prices().execute_on_dest_context();
        let last_price_ts: u64 = self.proxy(pool_address.clone()).get_last_price_ts().execute_on_dest_context();
        let ma_half_time: u64 = self.proxy(pool_address.clone()).get_ma_half_time().execute_on_dest_context();
        let xcp_profit: BigUint<Self::Api> = self.proxy(pool_address.clone()).get_xcp_profit().execute_on_dest_context();
        let xcp_profit_a: BigUint<Self::Api> = self.proxy(pool_address.clone()).get_xcp_profit_a().execute_on_dest_context();
        let is_not_adjusted: bool = self.proxy(pool_address.clone()).is_not_adjusted().execute_on_dest_context();
        let mut real_reserves: ManagedVec<BigUint> = ManagedVec::new();
        real_reserves.push(self.blockchain().get_esdt_balance(&pool_address.clone(), &token_0.clone(), 0));
        real_reserves.push(self.blockchain().get_esdt_balance(&pool_address.clone(), &token_1.clone(), 0));
        AshswapV2View {
            total_supply,
            reserves,
            real_reserves,
            amp_factor,
            gamma,
            virtual_price,
            price_oracle,
            price_scale,
            future_a_gamma_time,
            initial_a_gamma_time,
            d,
            mid_fee,
            out_fee,
            fee_gamma,
            state,
            allowed_extra_profit,
            adjustment_step,
            admin_fee,
            last_prices,
            last_price_ts,
            ma_half_time,
            xcp_profit,
            xcp_profit_a,
            is_not_adjusted,
        }
    }
}
