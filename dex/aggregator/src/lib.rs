#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use common_errors::*;
use common_structs::*;
use core::ops::Deref;

#[derive(TypeAbi, TopEncode)]
pub struct AggregatorEvent<M: ManagedTypeApi> {
    payment_in: ManagedVec<M, EsdtTokenPayment<M>>,
    payment_out: ManagedVec<M, EsdtTokenPayment<M>>,
}

#[multiversx_sc::contract]
pub trait AggregatorContract: token_send::TokenSendModule {
    #[init]
    fn init(&self) {}

    fn _find_token_in_vec(&self, tokens: &ManagedVec<TokenAmount<Self::Api>>, token_id: &TokenIdentifier) -> Option<usize> {
        for (index, token) in tokens.into_iter().enumerate() {
            if token_id == &token.token { return Some(index); }
        }
        None
    }

    fn _exchange(&self, vaults: &mut ManagedVec<TokenAmount<Self::Api>>, step: AggregatorStep<Self::Api>) {
        let index_in_opt = self._find_token_in_vec(vaults, &step.token_in);
        require!(index_in_opt.is_some(), ERROR_INVALID_TOKEN_IN);
        let index_in = index_in_opt.unwrap();

        let mut amount_in = vaults.get(index_in).amount;
        if step.amount_in > 0u64 {
            require!(amount_in >= step.amount_in, ERROR_INVALID_AMOUNT_IN);
            let remaining_amount = &amount_in - &step.amount_in;
            amount_in = step.amount_in;

            if remaining_amount > 0 {
                _ = vaults.set(index_in, &TokenAmount::new(step.token_in.clone(), remaining_amount));
            } else { vaults.remove(index_in); }
        } else { vaults.remove(index_in); }

        let mut payments = ManagedVec::new();
        payments.push(EsdtTokenPayment::new(step.token_in, 0, amount_in));

        let sc_address = self.blockchain().get_sc_address();
        let amount_before = self.blockchain().get_esdt_balance(&sc_address, &step.token_out, 0);

        let mut contract_call = self.send().contract_call::<()>(step.pool_address, step.function_name);
        for arg in step.arguments.into_iter() {
            contract_call.push_raw_argument(arg);
        }
        let _: IgnoreValue = contract_call.with_multi_token_transfer(payments).execute_on_dest_context();

        let amount_after = self.blockchain().get_esdt_balance(&sc_address, &step.token_out, 0);
        let mut amount_out = amount_after - amount_before;
        require!(amount_out > 0, ERROR_ZERO_AMOUNT);

        let index_out_opt = self._find_token_in_vec(vaults, &step.token_out);
        if let Some(index_out) = index_out_opt {
            amount_out += vaults.get(index_out).amount;
            _ = vaults.set(index_out, &TokenAmount::new(step.token_out, amount_out));
        } else {
            vaults.push(TokenAmount::new(step.token_out, amount_out));
        }
    }

    #[payable("*")]
    #[endpoint]
    fn aggregate(&self, steps: ManagedVec<AggregatorStep<Self::Api>>, limits: MultiValueEncoded<TokenAmount<Self::Api>>) -> ManagedVec<EsdtTokenPayment> {
        let payments = self.call_value().all_esdt_transfers();
        require!(!payments.is_empty(), ERROR_EMPTY_PAYMENTS);

        let mut vaults = ManagedVec::new();
        for payment in payments.into_iter() {
            require!(payment.token_nonce == 0, ERROR_ZERO_TOKEN_NONCE);
            require!(payment.amount > 0u64, ERROR_ZERO_AMOUNT);

            let index_opt = self._find_token_in_vec(&vaults, &payment.token_identifier);
            if let Some(index) = index_opt {
                let total_amount = &vaults.get(index).amount + &payment.amount;
                _ = vaults.set(index, &TokenAmount::new(payment.token_identifier, total_amount));
            } else { vaults.push(TokenAmount::new(payment.token_identifier, payment.amount)); }
        }

        let limits = limits.to_vec();
        let mut results = ManagedVec::new();
        for step in steps.into_iter() {
            require!(step.pool_address != self.blockchain().get_sc_address(), ERROR_INVALID_POOL_ADDR);
            self._exchange(&mut vaults, step);
        }

        require!(vaults.len() == limits.len(), ERROR_OUTPUT_LEN_MISMATCH);
        for vault in vaults.into_iter() {
            let index_opt = self._find_token_in_vec(&limits, &vault.token);
            require!(index_opt.is_some(), ERROR_INVALID_TOKEN_IN);
            let index = index_opt.unwrap();

            require!(vault.amount >= limits.get(index).amount, ERROR_SLIPPAGE_SCREW_YOU);
            results.push(EsdtTokenPayment::new(vault.token, 0, vault.amount));
        }

        let caller = self.blockchain().get_caller();
        let payment_out = self.send_multiple_tokens_if_not_zero(&caller, &results);
        self.aggregate_event(&caller, AggregatorEvent { payment_in: payments.deref().clone(), payment_out: payment_out.clone() });
        payment_out
    }

    #[event("aggregate_event")]
    fn aggregate_event(
        &self,
        #[indexed] caller: &ManagedAddress,
        aggregate: AggregatorEvent<Self::Api>,
    );
}
