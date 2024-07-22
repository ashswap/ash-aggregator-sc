multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::proxy]
pub trait WrapperProxy {
    #[view(getShareToAssetsPrice)]
    fn get_share_to_assets_price(&self) -> BigUint;
}
