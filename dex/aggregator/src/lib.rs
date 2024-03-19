#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use common_errors::*;
use common_structs::*;

use fee::ProxyTrait as _;

#[derive(TypeAbi, TopEncode)]
pub struct AggregatorEvent<M: ManagedTypeApi> {
    payment_in: EgldOrEsdtTokenPayment<M>,
    payment_out: EgldOrEsdtTokenPayment<M>,
}

#[multiversx_sc::contract]
pub trait AggregatorContract: token_send::TokenSendModule {
    #[init]
    fn init(&self, fee_address: ManagedAddress) {
        self.fee_address().set(&fee_address);
    }

    #[payable("*")]
    #[endpoint(aggregate)]
    fn aggregate(
            &self, 
            token_in: EgldOrEsdtTokenIdentifier,
            token_out: EgldOrEsdtTokenIdentifier,
            min_amount_out: BigUint, 
            steps: ManagedVec<AggregatorStep<Self::Api>>,
            protocol: OptionalValue<ManagedAddress>
        ) -> EgldOrEsdtTokenPayment {
        let payment = self.call_value().egld_or_single_esdt();
        require!(payment.amount > 0, ERROR_ZERO_AMOUNT);
        require!(payment.token_identifier == token_in, ERROR_INVALID_TOKEN_IN );
        require!(token_in != token_out, ERROR_SAME_TOKEN);

        let mut amount_in = payment.amount.clone();

        match protocol {
            OptionalValue::Some(protocol_addr) => {
                let fee_sc_address = self.fee_address().get();
                let (ash_fee, protocol_fee) = self.fee_contract_proxy(fee_sc_address.clone()).calculate_fee(&amount_in, &protocol_addr).execute_on_dest_context_readonly::<(BigUint, BigUint)>();
                let _: IgnoreValue = self
                    .fee_contract_proxy(fee_sc_address)
                    .charge_fee(&protocol_addr, &ash_fee, &protocol_fee)
                    .with_egld_or_single_esdt_transfer(EgldOrEsdtTokenPayment::new(payment.token_identifier.clone(), 0, &ash_fee + &protocol_fee))
                    .execute_on_dest_context();
                amount_in -= ash_fee + protocol_fee;
            }
            OptionalValue::None => {}
        }
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
            let mut contract_call = self.send().contract_call::<()>(step.pool_address, step.function_name);
            for arg in step.arguments.into_iter() {
                contract_call.push_raw_argument(arg);
            }
            let payment = EgldOrEsdtTokenPayment::new(step.token_in, 0, amount_in_step);
            let before_balance = self.blockchain().get_sc_balance(&step.token_out, 0);
            let _: IgnoreValue = contract_call.with_egld_or_single_esdt_transfer(payment).execute_on_dest_context();
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
        self.send().direct_non_zero(&caller, &token_out, 0, &final_amount_out);
        let payment_out = EgldOrEsdtTokenPayment::new(token_out, 0, final_amount_out.clone());
        self.aggregate_event(&caller, AggregatorEvent { payment_in: payment, payment_out: payment_out.clone() });
        payment_out
    }

    #[event("aggregate_event")]
    fn aggregate_event(
        &self,
        #[indexed] caller: &ManagedAddress,
        aggregate: AggregatorEvent<Self::Api>,
    );

    #[view(getFeeAddress)]
    #[storage_mapper("fee_address")]
    fn fee_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[proxy]
    fn fee_contract_proxy(&self, fee_sc_address: ManagedAddress) -> fee::Proxy<Self::Api>;
}
