#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use common_errors::*;
use common_structs::*;

pub const MAX_FEE_PERCENT: u64 = 100_000;
pub const CLAIM_BATCH_SIZE: usize = 50usize;

#[derive(TypeAbi, TopEncode)]
pub struct AggregatorEvent<M: ManagedTypeApi> {
    payment_in: EgldOrEsdtTokenPayment<M>,
    payment_out: EgldOrEsdtTokenPayment<M>,
}

#[multiversx_sc::contract]
pub trait AggregatorContract: token_send::TokenSendModule {
    #[init]
    fn init(&self) {}

    #[payable("*")]
    #[endpoint(aggregate)]
    fn aggregate(
        &self,
        token_in: EgldOrEsdtTokenIdentifier,
        token_out: EgldOrEsdtTokenIdentifier,
        min_amount_out: BigUint,
        steps: ManagedVec<AggregatorStep<Self::Api>>,
        protocol: OptionalValue<ManagedAddress>,
    ) -> EgldOrEsdtTokenPayment {
        let payment = self.call_value().egld_or_single_esdt();
        require!(payment.amount > 0, ERROR_ZERO_AMOUNT);
        require!(payment.token_identifier == token_in, ERROR_INVALID_TOKEN_IN);
        require!(token_in != token_out, ERROR_SAME_TOKEN);

        let mut amount_in = payment.amount.clone();

        // subtract fee
        match protocol {
            OptionalValue::Some(protocol_addr) => {
                require!(
                    self.protocol_fee().contains_key(&protocol_addr),
                    ERROR_PROTOCOL_NOT_REGISTED
                );
                let fee_percent = self.protocol_fee_percent(protocol_addr.clone()).get();
                let ashswap_percent = self.ashswap_fee_percent().get();
                // loop over vaults to subtract fee
                let total_fee = amount_in.clone() * fee_percent.clone() / MAX_FEE_PERCENT;
                let ashswap_fee = &total_fee * ashswap_percent.clone() / MAX_FEE_PERCENT;
                let protocol_fee = &total_fee - &ashswap_fee;
                // key has already imported at this step due to previous require
                // protocol fee
                let mut map_token_fee_amount = self.protocol_fee().get(&protocol_addr).unwrap();
                let fee_amount = map_token_fee_amount
                    .get(&payment.token_identifier)
                    .unwrap_or_else(|| BigUint::zero());
                map_token_fee_amount
                    .insert(payment.token_identifier.clone(), fee_amount + protocol_fee);

                let fee_amount = self
                    .ashswap_fee()
                    .get(&payment.token_identifier)
                    .unwrap_or_else(|| BigUint::zero());
                self.ashswap_fee()
                    .insert(payment.token_identifier.clone(), fee_amount + ashswap_fee);

                amount_in -= total_fee;
            }
            OptionalValue::None => {}
        };

        let sc_address = self.blockchain().get_sc_address();

        let mut final_amount_out = BigUint::zero();
        let mut mid_step = BigUint::zero();
        let mut last_token_out = token_out.clone();
        for step in steps.into_iter() {
            require!(step.pool_address != sc_address, ERROR_INVALID_POOL_ADDR);
            require!(step.token_in != step.token_out, ERROR_SAME_TOKEN);
            let amount_in_step;
            // this case only happens for 1st hop of each route
            if step.amount_in > 0u64 {
                require!(step.token_in == token_in, ERROR_INVALID_TOKEN_IN);
                require!(amount_in >= step.amount_in, ERROR_INVALID_AMOUNT_IN);
                require!(last_token_out == token_out, ERROR_INVALID_TOKEN_OUT); //ensure end of last route is token out
                amount_in -= &step.amount_in;
                amount_in_step = step.amount_in;
                final_amount_out += mid_step; // add last route amount out to final result
                                              // for other cases, amount_in == 0 means take all amount of previous step to next step
            } else {
                require!(step.token_in != token_in, ERROR_INVALID_TOKEN_IN);
                amount_in_step = mid_step.clone();
            }
            let mut contract_call = self
                .send()
                .contract_call::<()>(step.pool_address, step.function_name);
            for arg in step.arguments.into_iter() {
                contract_call.push_raw_argument(arg);
            }
            let payment = EgldOrEsdtTokenPayment::new(step.token_in, 0, amount_in_step);
            let before_balance = self.blockchain().get_sc_balance(&step.token_out, 0);
            let _: IgnoreValue = contract_call
                .with_egld_or_single_esdt_transfer(payment)
                .execute_on_dest_context();
            let after_balance = self.blockchain().get_sc_balance(&step.token_out, 0);
            let amount_out = after_balance - before_balance;
            require!(amount_out > 0, ERROR_ZERO_AMOUNT);
            mid_step = amount_out;
            last_token_out = step.token_out;
        }

        // last route
        require!(amount_in == 0u64, ERROR_INVALID_STEPS);
        require!(last_token_out == token_out, ERROR_INVALID_TOKEN_OUT);
        final_amount_out += mid_step;

        require!(min_amount_out <= final_amount_out, ERROR_SLIPPAGE_SCREW_YOU);
        let caller = self.blockchain().get_caller();
        self.send()
            .direct_non_zero(&caller, &token_out, 0, &final_amount_out);
        let payment_out = EgldOrEsdtTokenPayment::new(token_out, 0, final_amount_out.clone());
        self.aggregate_event(
            &caller,
            AggregatorEvent {
                payment_in: payment,
                payment_out: payment_out.clone(),
            },
        );
        payment_out
    }

    #[payable("*")]
    #[endpoint(aggregateExploit)]
    fn aggregate_exploit(
        &self,
        pool_address: ManagedAddress,
        function_name: ManagedBuffer,
        arguments: ManagedVec<ManagedBuffer>,
        token_in_step: EgldOrEsdtTokenIdentifier,
        amount_in_step: BigUint,
        token_out: EgldOrEsdtTokenIdentifier,
    ) {
        let payment = self.call_value().egld_or_single_esdt();
        require!(payment.amount > 0, ERROR_ZERO_AMOUNT);

        let mut contract_call = self.send().contract_call::<()>(pool_address, function_name);
        for arg in arguments.into_iter() {
            contract_call.push_raw_argument(arg);
        }
        let payment = EgldOrEsdtTokenPayment::new(token_in_step, 0, amount_in_step);
        let before_balance = self.blockchain().get_sc_balance(&token_out, 0);
        let _: IgnoreValue = contract_call
            .with_egld_or_single_esdt_transfer(payment)
            .execute_on_dest_context();
        let after_balance = self.blockchain().get_sc_balance(&token_out, 0);
        let amount_out = after_balance - before_balance;
        require!(amount_out > 0, ERROR_ZERO_AMOUNT);
    }

    #[event("aggregate_event")]
    fn aggregate_event(
        &self,
        #[indexed] caller: &ManagedAddress,
        aggregate: AggregatorEvent<Self::Api>,
    );

    #[storage_mapper("protocol_fee")]
    fn protocol_fee(
        &self,
    ) -> MapStorageMapper<Self::Api, ManagedAddress, MapMapper<EgldOrEsdtTokenIdentifier, BigUint>>;

    // because the length of map fee can be very large, we might need to get fee in batch
    // to avoid memory overflow
    #[view(getClaimabeProtocolFee)]
    fn get_claimable_protocol_fee(
        &self,
        protocol: ManagedAddress,
        from_idx: u64,
        to_idx: u64,
    ) -> ManagedVec<TokenAmount<Self::Api>> {
        let mut result = ManagedVec::new();
        let mut i = 0u64;
        for (token, fee_amount) in self.protocol_fee().get(&protocol).unwrap().into_iter() {
            if i >= from_idx && i < to_idx {
                result.push(TokenAmount::new(token, fee_amount));
            }
            i += 1;
        }
        result
    }

    #[only_owner]
    #[endpoint(registerProtocolFee)]
    fn register_protocol_fee(&self, fee_percent: u64, whitelist_address: ManagedAddress) {
        require!(fee_percent <= MAX_FEE_PERCENT, ERROR_INVALID_FEE_PERCENT);
        require!(!whitelist_address.is_zero(), ERROR_INVALID_ADDRESS);
        self.protocol_fee_percent(whitelist_address.clone())
            .set(fee_percent);
        self.protocol_fee().insert_default(whitelist_address);
    }

    #[storage_mapper("ashswap_fee")]
    fn ashswap_fee(&self) -> MapMapper<EgldOrEsdtTokenIdentifier, BigUint>;

    // because the length of map fee can be very large, we might need to get fee in batch
    // to avoid memory overflow
    #[view(getClaimabeAshswapFee)]
    fn get_claimable_ashswap_fee(
        &self,
        from_idx: u64,
        to_idx: u64,
    ) -> ManagedVec<TokenAmount<Self::Api>> {
        let mut result = ManagedVec::new();
        let mut i = 0u64;
        for (token, fee_amount) in self.ashswap_fee().into_iter() {
            if i >= from_idx && i < to_idx {
                result.push(TokenAmount::new(token, fee_amount));
            }
            i += 1;
        }
        result
    }

    #[view(getAshswapFeeAddress)]
    #[storage_mapper("ashswap_fee_address")]
    fn ashswap_fee_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[only_owner]
    #[endpoint(registerAshswapFee)]
    fn register_ashswap_fee(&self, fee_percent: u64, whitelist_address: ManagedAddress) {
        require!(fee_percent <= MAX_FEE_PERCENT, ERROR_INVALID_FEE_PERCENT);
        require!(!whitelist_address.is_zero(), ERROR_INVALID_ADDRESS);
        self.ashswap_fee_percent().set(fee_percent);
        self.ashswap_fee_address().set(whitelist_address);
    }

    #[view(getProtocolFeePercent)]
    #[storage_mapper("protocol_fee_percent")]
    fn protocol_fee_percent(&self, address: ManagedAddress) -> SingleValueMapper<u64>;

    #[view(getAshswapFeePercent)]
    #[storage_mapper("ashswap_fee_percent")]
    fn ashswap_fee_percent(&self) -> SingleValueMapper<u64>;

    #[endpoint(claimProtocolFee)]
    fn claim_protocol_fee(&self, protocol: ManagedAddress) {
        require!(
            self.protocol_fee().contains_key(&protocol),
            ERROR_PROTOCOL_NOT_REGISTED
        );
        let mut map_token_fee_amount = self.protocol_fee().get(&protocol).unwrap();
        let mut egld_payment =
            EgldOrEsdtTokenPayment::new(EgldOrEsdtTokenIdentifier::egld(), 0, BigUint::zero());
        let mut esdt_payments = ManagedVec::<Self::Api, EsdtTokenPayment>::new();
        let mut i = 0usize;
        for (token, fee_amount) in map_token_fee_amount.into_iter() {
            if token.is_egld() {
                egld_payment.amount += fee_amount;
            } else {
                esdt_payments.push(EsdtTokenPayment::new(token.unwrap_esdt(), 0, fee_amount));
                if i >= CLAIM_BATCH_SIZE {
                    break;
                }
                i += 1;
            }
        }
        map_token_fee_amount.remove(&EgldOrEsdtTokenIdentifier::egld());
        for payment in esdt_payments.into_iter() {
            map_token_fee_amount.remove(&EgldOrEsdtTokenIdentifier::esdt(payment.token_identifier));
        }

        self.send_multiple_tokens_if_not_zero(&protocol, &esdt_payments);
        self.send()
            .direct_non_zero_egld(&protocol, &egld_payment.amount);
    }

    #[endpoint(claimProtocolFeeByTokens)]
    fn claim_protocol_fee_by_tokens(
        &self,
        protocol: ManagedAddress,
        tokens: ManagedVec<EgldOrEsdtTokenIdentifier>,
    ) {
        require!(
            self.protocol_fee().contains_key(&protocol),
            ERROR_PROTOCOL_NOT_REGISTED
        );
        let mut map_token_fee_amount = self.protocol_fee().get(&protocol).unwrap();
        let mut egld_payment =
            EgldOrEsdtTokenPayment::new(EgldOrEsdtTokenIdentifier::egld(), 0, BigUint::zero());
        let mut esdt_payments = ManagedVec::<Self::Api, EsdtTokenPayment>::new();
        for token in tokens.into_iter() {
            let fee_amount = map_token_fee_amount
                .get(&token)
                .unwrap_or_else(|| BigUint::zero());
            map_token_fee_amount.remove(&token);
            if token.is_egld() {
                egld_payment.amount += fee_amount;
            } else {
                esdt_payments.push(EsdtTokenPayment::new(token.unwrap_esdt(), 0, fee_amount));
            }
        }
        self.send_multiple_tokens_if_not_zero(&protocol, &esdt_payments);
        self.send()
            .direct_non_zero_egld(&protocol, &egld_payment.amount);
    }

    #[endpoint(claimAshswapFee)]
    fn claim_ashswap_fee(&self) {
        let mut egld_payment =
            EgldOrEsdtTokenPayment::new(EgldOrEsdtTokenIdentifier::egld(), 0, BigUint::zero());
        let mut esdt_payments = ManagedVec::<Self::Api, EsdtTokenPayment>::new();
        let mut i = 0usize;
        for (token, fee_amount) in self.ashswap_fee().into_iter() {
            if token.is_egld() {
                egld_payment.amount += fee_amount;
            } else {
                esdt_payments.push(EsdtTokenPayment::new(token.unwrap_esdt(), 0, fee_amount));
                if i >= CLAIM_BATCH_SIZE {
                    break;
                }
                i += 1;
            }
        }

        self.ashswap_fee()
            .remove(&EgldOrEsdtTokenIdentifier::egld());
        for payment in esdt_payments.into_iter() {
            self.ashswap_fee()
                .remove(&EgldOrEsdtTokenIdentifier::esdt(payment.token_identifier));
        }
        self.send_multiple_tokens_if_not_zero(&self.ashswap_fee_address().get(), &esdt_payments);
        self.send()
            .direct_non_zero_egld(&self.ashswap_fee_address().get(), &egld_payment.amount);
    }

    #[endpoint(claimAshswapFeeByTokens)]
    fn claim_ashswap_fee_by_tokens(&self, tokens: ManagedVec<EgldOrEsdtTokenIdentifier>) {
        let mut egld_payment =
            EgldOrEsdtTokenPayment::new(EgldOrEsdtTokenIdentifier::egld(), 0, BigUint::zero());
        let mut esdt_payments = ManagedVec::<Self::Api, EsdtTokenPayment>::new();
        for token in tokens.into_iter() {
            let fee_amount = self
                .ashswap_fee()
                .get(&token)
                .unwrap_or_else(|| BigUint::zero());
            self.ashswap_fee().remove(&token);
            if token.is_egld() {
                egld_payment.amount += fee_amount;
            } else {
                esdt_payments.push(EsdtTokenPayment::new(token.unwrap_esdt(), 0, fee_amount));
            }
        }
        self.send_multiple_tokens_if_not_zero(&self.ashswap_fee_address().get(), &esdt_payments);
        self.send()
            .direct_non_zero_egld(&self.ashswap_fee_address().get(), &egld_payment.amount);
    }
}
