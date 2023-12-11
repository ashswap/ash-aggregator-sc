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
    fn send_multiple_tokens_if_not_zero_support_egld(&self, destination: &ManagedAddress, payments: &ManagedVec<EsdtTokenPayment>) -> (ManagedVec<EsdtTokenPayment>, EsdtTokenPayment) {
        let mut non_zero_payments = ManagedVec::new();
        let wrapped_egld_token_id = self.egld_wrapped_token_id().get();
        let mut unwrap_egld_payment = EsdtTokenPayment::new(wrapped_egld_token_id.clone(), 0, 0u64.into());
        for payment in payments {
            if payment.amount > 0u64 {
                if payment.token_identifier == wrapped_egld_token_id {
                    unwrap_egld_payment.amount += payment.amount;
                } else {
                    non_zero_payments.push(payment);
                }
            }
        }
        if !non_zero_payments.is_empty() {
            self.send().direct_multi(destination, &non_zero_payments);
        }
        if unwrap_egld_payment.amount > 0u64 {
            self.unwrap_egld(unwrap_egld_payment.clone());
            self.send().direct_egld(destination, &unwrap_egld_payment.amount);
        }

        (non_zero_payments, unwrap_egld_payment)
    }

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

    fn wrap_egld(&self, amount: BigUint) -> EsdtTokenPayment {
        let _: IgnoreValue = self.egld_wrapper_proxy(self.egld_wrapper_address().get())
            .wrap_egld().with_egld_transfer(amount.clone()).execute_on_dest_context();
        EsdtTokenPayment::new(self.egld_wrapped_token_id().get(), 0, amount)
    }

    fn unwrap_egld(&self, payment: EsdtTokenPayment) {
        let _: IgnoreValue = self.egld_wrapper_proxy(self.egld_wrapper_address().get())
            .unwrap_egld().with_esdt_transfer(payment).execute_on_dest_context();
    }

    #[proxy]
    fn egld_wrapper_proxy(&self, to: ManagedAddress) -> proxy::Proxy<Self::Api>;

    #[view(getEgldWrapperAddress)]
    #[storage_mapper("egld_wrapper_address")]
    fn egld_wrapper_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getEgldWrappedTokenId)]
    #[storage_mapper("egld_wrapped_token_id")]
    fn egld_wrapped_token_id(&self) -> SingleValueMapper<TokenIdentifier>;
}
