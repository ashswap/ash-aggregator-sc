#![no_std]

pub mod proxy;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, TypeAbi, Clone)]
pub struct EgldWrapperOption<M: ManagedTypeApi> {
    pub wrapped_egld_token_id: TokenIdentifier<M>,
    pub egld_return: bool,
}

#[multiversx_sc::module]
pub trait TokenSendModule {

    fn send_multiple_tokens_if_not_zero(&self, destination: &ManagedAddress, payments: &ManagedVec<EsdtTokenPayment>) -> ManagedVec<EsdtTokenPayment> {
        let mut non_zero_payments = ManagedVec::new();
        for payment in payments {
            if payment.amount > 0u64 {
                non_zero_payments.push(payment);
            }
        }

        if !non_zero_payments.is_empty() {
            self.send().direct_multi(destination, &non_zero_payments);
        }
        non_zero_payments
    }

    fn send_tokens_non_zero(&self, to: &ManagedAddress, token_id: &TokenIdentifier, nonce: u64, amount: &BigUint) -> Option<EsdtTokenPayment> {
        if amount > &0 {
            self.send().direct_esdt(to, token_id, nonce, amount);
            return Some(EsdtTokenPayment::new(token_id.clone(), nonce, amount.clone()));
        }
        None
    }
}
