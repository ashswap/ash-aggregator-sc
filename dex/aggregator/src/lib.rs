#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use common_errors::*;
use common_structs::*;

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
            _ = vaults.set(index_in, &TokenAmount::new(step.token_in.clone(), remaining_amount));
            amount_in = step.amount_in;
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
            vaults.push(TokenAmount::new(payment.token_identifier, payment.amount));
        }

        let limits = limits.to_vec();
        let mut payments = ManagedVec::new();
        for step in steps.into_iter() {
            self._exchange(&mut vaults, step);
        }

        for vault in vaults.into_iter() {
            let mut limit_amount = BigUint::zero();
            let index_opt = self._find_token_in_vec(&limits, &vault.token);
            if let Some(index) = index_opt { limit_amount = limits.get(index).amount; }

            require!(vault.amount >= limit_amount, ERROR_SLIPPAGE_SCREW_YOU);
            payments.push(EsdtTokenPayment::new(vault.token, 0, vault.amount));
        }

        self.send_multiple_tokens_if_not_zero(&self.blockchain().get_caller(), &payments)
    }
}
