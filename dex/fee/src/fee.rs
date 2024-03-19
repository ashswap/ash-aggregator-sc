#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;

use common_errors::*;
use common_structs::*;

pub const MAX_FEE_PERCENT: u64 = 100_000;
pub const CLAIM_BATCH_SIZE: usize = 50usize;

#[multiversx_sc::contract]
pub trait Fee: token_send::TokenSendModule {
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}


    #[view(calculateFee)]
    fn calculate_fee(&self, amount_in: &BigUint, protocol_addr: &ManagedAddress) -> (BigUint, BigUint) {
        require!(self.protocol_fee().contains_key(protocol_addr), ERROR_PROTOCOL_NOT_REGISTED);
        let fee_percent = self.protocol_fee_percent(protocol_addr.clone()).get();
        let ashswap_percent = self.ashswap_fee_percent().get();
        // loop over vaults to subtract fee
        let total_fee = amount_in * fee_percent / MAX_FEE_PERCENT;
        let ashswap_fee = &total_fee * ashswap_percent / MAX_FEE_PERCENT;
        let protocol_fee = &total_fee - &ashswap_fee;
        return (ashswap_fee, protocol_fee);
    }

    #[payable("*")]
    #[endpoint(chargeFee)]
    fn charge_fee(&self, protocol: &ManagedAddress, ash_fee: &BigUint, protocol_fee: &BigUint) {
        let payment = self.call_value().egld_or_single_esdt();
        require!(payment.amount > 0, ERROR_ZERO_AMOUNT);
        require!(payment.amount == ash_fee + protocol_fee, ERROR_INSUFFICIENT_AMOUNT);
        require!(self.protocol_fee().contains_key(&protocol), ERROR_PROTOCOL_NOT_REGISTED);
        let mut map_token_fee_amount = self.protocol_fee().get(&protocol).unwrap();
        let fee_amount = map_token_fee_amount.get(&payment.token_identifier).unwrap_or_else(|| BigUint::zero());
        map_token_fee_amount.insert(payment.token_identifier.clone(), fee_amount + protocol_fee);

        let fee_amount = self.ashswap_fee().get(&payment.token_identifier).unwrap_or_else(|| BigUint::zero());
        self.ashswap_fee().insert(payment.token_identifier.clone(), fee_amount + ash_fee);
    }

    #[storage_mapper("protocol_fee")]
    fn protocol_fee(&self) -> MapStorageMapper<Self::Api, ManagedAddress, MapMapper<EgldOrEsdtTokenIdentifier, BigUint>>;

    // because the length of map fee can be very large, we might need to get fee in batch
    // to avoid memory overflow
    #[view(getClaimabeProtocolFee)]
    fn get_claimable_protocol_fee(&self, protocol: ManagedAddress, from_idx: u64, to_idx: u64) -> ManagedVec<TokenAmount<Self::Api>> {
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
        self.protocol_fee_percent(whitelist_address.clone()).set(fee_percent);
        self.protocol_fee().insert_default(whitelist_address);
    }

    #[storage_mapper("ashswap_fee")]
    fn ashswap_fee(&self) -> MapMapper<EgldOrEsdtTokenIdentifier, BigUint>;

    // because the length of map fee can be very large, we might need to get fee in batch
    // to avoid memory overflow
    #[view(getClaimabeAshswapFee)]
    fn get_claimable_ashswap_fee(&self, from_idx: u64, to_idx: u64) -> ManagedVec<TokenAmount<Self::Api>> {
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
        require!(self.protocol_fee().contains_key(&protocol), ERROR_PROTOCOL_NOT_REGISTED);
        let mut map_token_fee_amount = self.protocol_fee().get(&protocol).unwrap();
        let mut egld_payment = EgldOrEsdtTokenPayment::new(EgldOrEsdtTokenIdentifier::egld(), 0, BigUint::zero());
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
        self.send().direct_non_zero_egld(&protocol, &egld_payment.amount);
    }

    #[endpoint(claimProtocolFeeByTokens)]
    fn claim_protocol_fee_by_tokens(&self, protocol: ManagedAddress, tokens: ManagedVec<EgldOrEsdtTokenIdentifier>) {
        require!(self.protocol_fee().contains_key(&protocol), ERROR_PROTOCOL_NOT_REGISTED);
        let mut map_token_fee_amount = self.protocol_fee().get(&protocol).unwrap();
        let mut egld_payment = EgldOrEsdtTokenPayment::new(EgldOrEsdtTokenIdentifier::egld(), 0, BigUint::zero());
        let mut esdt_payments = ManagedVec::<Self::Api, EsdtTokenPayment>::new();
        for token in tokens.into_iter() {
            let fee_amount = map_token_fee_amount.get(&token).unwrap_or_else(|| BigUint::zero());
            map_token_fee_amount.remove(&token);
            if token.is_egld() {
                egld_payment.amount += fee_amount;
            } else {
                esdt_payments.push(EsdtTokenPayment::new(token.unwrap_esdt(), 0, fee_amount));
            }
        }
        self.send_multiple_tokens_if_not_zero(&protocol, &esdt_payments);
        self.send().direct_non_zero_egld(&protocol, &egld_payment.amount);
    }

    #[endpoint(claimAshswapFee)]
    fn claim_ashswap_fee(&self) {
        let mut egld_payment = EgldOrEsdtTokenPayment::new(EgldOrEsdtTokenIdentifier::egld(), 0, BigUint::zero());
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

        self.ashswap_fee().remove(&EgldOrEsdtTokenIdentifier::egld());
        for payment in esdt_payments.into_iter() {
            self.ashswap_fee().remove(&EgldOrEsdtTokenIdentifier::esdt(payment.token_identifier));
        }
        self.send_multiple_tokens_if_not_zero(&self.ashswap_fee_address().get(), &esdt_payments);
        self.send().direct_non_zero_egld(&self.ashswap_fee_address().get(), &egld_payment.amount);
    }

    #[endpoint(claimAshswapFeeByTokens)]
    fn claim_ashswap_fee_by_tokens(&self, tokens: ManagedVec<EgldOrEsdtTokenIdentifier>) {
        let mut egld_payment = EgldOrEsdtTokenPayment::new(EgldOrEsdtTokenIdentifier::egld(), 0, BigUint::zero());
        let mut esdt_payments = ManagedVec::<Self::Api, EsdtTokenPayment>::new();
        for token in tokens.into_iter() {
            let fee_amount = self.ashswap_fee().get(&token).unwrap_or_else(|| BigUint::zero());
            self.ashswap_fee().remove(&token);
            if token.is_egld() {
                egld_payment.amount += fee_amount;
            } else {
                esdt_payments.push(EsdtTokenPayment::new(token.unwrap_esdt(), 0, fee_amount));
            }
        }
        self.send_multiple_tokens_if_not_zero(&self.ashswap_fee_address().get(), &esdt_payments);
        self.send().direct_non_zero_egld(&self.ashswap_fee_address().get(), &egld_payment.amount);
    }
}
