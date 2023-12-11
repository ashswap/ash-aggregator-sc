#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub mod ashswapv1;
pub mod ashswapv2;
pub mod jexexchange;
pub mod onedex;
pub mod xexchange;

#[multiversx_sc::contract]
pub trait WrapperView:
    jexexchange::WrapperModule + onedex::WrapperModule + xexchange::WrapperModule
{
    #[init]
    fn init(&self) {}
}
