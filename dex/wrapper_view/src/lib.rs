#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub mod ashswapv1;
pub mod ashswapv2;
pub mod jexexchange;
pub mod onedex;
pub mod xexchange;
pub mod hatom;
pub mod tinder;
pub mod hatomliquidstaking;

#[multiversx_sc::contract]
pub trait WrapperView:
    ashswapv1::WrapperModule
    + ashswapv2::WrapperModule
    + jexexchange::WrapperModule
    + onedex::WrapperModule
    + xexchange::WrapperModule
    + hatom::WrapperModule
    + tinder::WrapperModule
    + hatomliquidstaking::WrapperModule
{
    #[init]
    fn init(&self) {}
}
