#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::contract]
pub trait EgldWrapperMock {
    #[init]
    fn init(&self, token_id: TokenIdentifier) {
        self.wrapped_egld_token_id().set(token_id);
    }

    #[payable("EGLD")]
    #[endpoint(wrapEgld)]
    fn wrap_egld(&self) -> EsdtTokenPayment<Self::Api> {
        let egld_amount: BigUint = self.call_value().egld_value().clone_value();
        let token_id = self.wrapped_egld_token_id().get();
        self.send().direct_esdt(&self.blockchain().get_caller(), &token_id, 0, &egld_amount);
        EsdtTokenPayment::new(token_id, 0, egld_amount)
    }

    #[payable("*")]
    #[endpoint(unwrapEgld)]
    fn unwrap_egld(&self) {
        let payment = self.call_value().single_esdt();
        require!(payment.token_identifier == self.wrapped_egld_token_id().get(), "Wrong token id");
        self.send().direct_egld(&self.blockchain().get_caller(), &payment.amount);
    }

    #[view(getWrappedEgldTokenId)]
    #[storage_mapper("wrapped_egld_token_id")]
    fn wrapped_egld_token_id(&self) -> SingleValueMapper<TokenIdentifier>;
}
