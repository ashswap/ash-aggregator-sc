#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use common_errors::*;
use common_structs::*;
use core::ops::Deref;

pub const MAX_FEE_PERCENT: u64 = 100_000;
pub const CLAIM_BATCH_SIZE: usize = 50usize;

#[derive(TypeAbi, TopEncode)]
pub struct AggregatorEvent<M: ManagedTypeApi> {
    payment_in: ManagedVec<M, EsdtTokenPayment<M>>,
    payment_out: ManagedVec<M, EsdtTokenPayment<M>>,
}

#[multiversx_sc::contract]
pub trait AggregatorContract: token_send::TokenSendModule {
    #[init]
    fn init(&self) {}

    fn _find_token_in_vault(&self, tokens: &ManagedVec<TokenAmount<Self::Api>>, token_id: &TokenIdentifier) -> Option<usize> {
        for (index, token) in tokens.into_iter().enumerate() {
            if token_id == &token.token { return Some(index); }
        }
        None
    }

    fn _upsert_vaults(&self, vaults: &mut ManagedVec<TokenAmount<Self::Api>>, token_id: &TokenIdentifier, amount: BigUint) {
        let index_opt = self._find_token_in_vault(vaults, token_id);
        if let Some(index) = index_opt {
            let total_amount = vaults.get(index).amount + amount;
            _ = vaults.set(index, &TokenAmount::new(token_id.clone(), total_amount));
        } else { vaults.push(TokenAmount::new(token_id.clone(), amount)); }
    }

    fn _exchange(&self, vaults: &mut ManagedVec<TokenAmount<Self::Api>>, step: AggregatorStep<Self::Api>) {
        let index_in_opt = self._find_token_in_vault(vaults, &step.token_in);
        require!(index_in_opt.is_some(), ERROR_INVALID_TOKEN_IN);
        let index_in = index_in_opt.unwrap();

        let mut amount_in = vaults.get(index_in).amount;
        if step.amount_in > 0u64 {
            require!(amount_in >= step.amount_in, ERROR_INVALID_AMOUNT_IN);
            let remaining_amount = &amount_in - &step.amount_in;
            _ = vaults.set(index_in, &TokenAmount::new(step.token_in.clone(), remaining_amount));
            amount_in = step.amount_in;
        } else { _ = vaults.set(index_in, &TokenAmount::new(step.token_in.clone(), BigUint::zero())); }

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
        let amount_out = amount_after - amount_before;
        require!(amount_out > 0, ERROR_ZERO_AMOUNT);
        self._upsert_vaults(vaults, &step.token_out, amount_out);
    }

    #[payable("*")]
    #[endpoint]
    fn aggregate(&self, steps: ManagedVec<AggregatorStep<Self::Api>>, limits: ManagedVec<TokenAmount<Self::Api>>, protocol: OptionalValue<ManagedAddress<Self::Api>>) -> ManagedVec<EsdtTokenPayment> {
        let payments = self.call_value().all_esdt_transfers();
        require!(!payments.is_empty(), ERROR_EMPTY_PAYMENTS);

        let mut vaults = ManagedVec::new();
        
        for payment in payments.into_iter() {
            require!(payment.token_nonce == 0, ERROR_ZERO_TOKEN_NONCE);
            require!(payment.amount > 0u64, ERROR_ZERO_AMOUNT);
            self._upsert_vaults(&mut vaults, &payment.token_identifier, payment.amount);
        }

        let is_charge_fee = protocol.is_some();
        let fee_percent: u64;
        let ashswap_percent: u64;
        let protocol_addr = match protocol {
            OptionalValue::Some(protocol_addr) => {
                require!(self.protocol_fee().contains_key(&protocol_addr), ERROR_PROTOCOL_NOT_REGISTED);
                fee_percent = self.protocol_fee_percent(protocol_addr.clone()).get();
                ashswap_percent = self.ashswap_fee_percent().get();
                protocol_addr
            },
            OptionalValue::None => {
                fee_percent = 0u64;
                ashswap_percent = 0u64;
                ManagedAddress::zero()
            },
        };

        if is_charge_fee {
           // loop over vaults to subtract fee
            for index in 0..vaults.len() {
                let vault = vaults.get(index).clone();
                let total_fee = vault.amount.clone() * fee_percent.clone() / MAX_FEE_PERCENT;
                let ashswap_fee = &total_fee * ashswap_percent.clone() / MAX_FEE_PERCENT;
                let protocol_fee = &total_fee - &ashswap_fee;
                // key has already imported at this step due to previous require
                // protocol fee
                let mut map_token_fee_amount = self.protocol_fee().get(&protocol_addr).unwrap();
                match map_token_fee_amount.get(&vault.token) {
                    Some(fee_amount) => {
                        _ = map_token_fee_amount.insert(vault.token.clone(), fee_amount + protocol_fee);
                    },
                    None => {
                        _ = map_token_fee_amount.insert(vault.token.clone(), protocol_fee.clone());
                    },
                }
                // ashswap fee
                match self.ashswap_fee().get(&vault.token) {
                    Some(fee_amount) => {
                        _ = self.ashswap_fee().insert(vault.token.clone(), fee_amount + ashswap_fee);
                    },
                    None => {
                        _ = self.ashswap_fee().insert(vault.token.clone(), ashswap_fee.clone());
                    },
                }
                _ = vaults.set(index, &TokenAmount::new(vault.token, vault.amount - total_fee));
            } 
        }
        


        let mut results = ManagedVec::new();
        let sc_address = self.blockchain().get_sc_address();

        for step in steps.into_iter() {
            require!(step.pool_address != sc_address, ERROR_INVALID_POOL_ADDR);
            self._exchange(&mut vaults, step);
        }

        require!(vaults.len() == limits.len(), ERROR_OUTPUT_LEN_MISMATCH);
        for limit in limits.into_iter() {
            let index_opt = self._find_token_in_vault(&vaults, &limit.token);
            require!(index_opt.is_some(), ERROR_INVALID_TOKEN_IN);
            let index = index_opt.unwrap();

            let vault = vaults.get(index);
            require!(vault.amount >= limit.amount, ERROR_SLIPPAGE_SCREW_YOU);
            results.push(EsdtTokenPayment::new(vault.token, 0, vault.amount));

            // remove index from vaults for de-duplicate limits
            vaults.remove(index);
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

    #[storage_mapper("protocol_fee")]
    fn protocol_fee(&self) -> MapStorageMapper<Self::Api, ManagedAddress, MapMapper<TokenIdentifier, BigUint>>;

    #[only_owner]
    #[endpoint(registerProtocolFee)]
    fn register_protocol_fee(&self, fee_percent: u64, whitelist_address: ManagedAddress) {
        self.protocol_fee_percent(whitelist_address.clone()).set(fee_percent);
        self.protocol_fee().insert_default(whitelist_address);
    }

    #[storage_mapper("ashswap_fee")]
    fn ashswap_fee(&self) -> MapMapper<TokenIdentifier, BigUint>;

    #[storage_mapper("ashswap_fee_address")]
    fn ashswap_fee_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[only_owner]
    #[endpoint(registerAshswapFee)]
    fn register_ashswap_fee(&self, fee_percent: u64, whitelist_address: ManagedAddress) {
        self.ashswap_fee_percent().set(fee_percent);
        self.ashswap_fee_address().set(whitelist_address);
    }

    #[view(getProtocolFeePercent)]
    #[storage_mapper("protocol_fee_percent")]
    fn protocol_fee_percent(&self, address: ManagedAddress) -> SingleValueMapper<u64>;

    #[view(getAshswapFeePercent)]
    #[storage_mapper("ashswap_fee_percent")]
    fn ashswap_fee_percent(&self) -> SingleValueMapper<u64>;

    #[endpoint]
    fn claim(&self, protocol: ManagedAddress<Self::Api>) {
        require!(self.protocol_fee().contains_key(&protocol), ERROR_PROTOCOL_NOT_REGISTED);
        let mut map_token_fee_amount = self.protocol_fee().get(&protocol).unwrap();
        let mut payments = ManagedVec::<Self::Api, EsdtTokenPayment<Self::Api>>::new();
        let mut i = 0usize;
        for (token, fee_amount) in map_token_fee_amount.into_iter() {
            payments.push(EsdtTokenPayment::new(token, 0, fee_amount));
            if i >= CLAIM_BATCH_SIZE {
                break;
            }
            i += 1;
        }
        for payment in payments.into_iter() {
            map_token_fee_amount.remove(&payment.token_identifier);
        }
        self.send_multiple_tokens_if_not_zero(&protocol, &payments);
    }

    #[endpoint]
    fn ashswap_claim(&self) {
        let mut payments = ManagedVec::<Self::Api, EsdtTokenPayment<Self::Api>>::new();
        let mut i = 0usize;
        for (token, fee_amount) in self.ashswap_fee().into_iter() {
            payments.push(EsdtTokenPayment::new(token, 0, fee_amount));
            if i >= CLAIM_BATCH_SIZE {
                break;
            }
            i += 1;
        }
        for payment in payments.into_iter() {
            self.ashswap_fee().remove(&payment.token_identifier);
        }
        self.send_multiple_tokens_if_not_zero(&self.ashswap_fee_address().get(), &payments);
    }
}
