multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::proxy]
pub trait EgldWrapper {
    #[payable("EGLD")]
    #[endpoint(wrapEgld)]
    fn wrap_egld(&self) -> EsdtTokenPayment<Self::Api>;

    #[payable("*")]
    #[endpoint(unwrapEgld)]
    fn unwrap_egld(&self);

    #[view(getWrappedEgldTokenId)]
    fn get_wrapped_egld_token_id(&self) -> TokenIdentifier;
}
