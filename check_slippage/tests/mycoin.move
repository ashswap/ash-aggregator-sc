#[test_only]
module check_slippage::my_coin {
    use sui::coin;

    public struct MY_COIN has drop {}

    fun init(witness: MY_COIN, ctx: &mut TxContext) {
        let (treasury, metadata) = coin::create_currency(
            witness,
            6,                // decimals
            b"MYC",           // symbol
            b"My Coin",       // name
            b"Don't ask why", // description
            option::none(),   // icon url
            ctx
        );

        // transfer the `TreasuryCap` to the sender, so they can mint and burn
        transfer::public_transfer(treasury, ctx.sender());

        // metadata is typically frozen after creation
        transfer::public_freeze_object(metadata);
    }
}
