multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::contract]
pub trait ProtocolMock {
    #[payable("*")]
    #[endpoint]
    fn exchange(&self, token_out: TokenIdentifier) {
        let payments = self.call_value().all_esdt_transfers();
        let amount_out = payments.get(0).amount * 95u64 / 100u64; // receive 95% amount
        self.send()
            .direct_esdt(&self.blockchain().get_caller(), &token_out, 0, &amount_out);
    }
}
