multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::contract]
pub trait ProtocolMock {
    #[payable("*")]
    #[endpoint]
    fn exchange(&self, token_out: EgldOrEsdtTokenIdentifier) {
        let payment = self.call_value().egld_or_single_esdt();
        let amount_out = payment.amount * 95u64 / 100u64; // receive 95% amount
        if token_out.is_egld() {
            self.send().direct_egld(&self.blockchain().get_caller(), &amount_out);
        } else {
            self.send().direct_esdt(&self.blockchain().get_caller(), &token_out.unwrap_esdt(), 0, &amount_out);
        }
    }
}
