// Define states for the second dialogue (SetPrice)
#[derive(Clone, Default)]
pub enum PriceState {
    #[default]
    Start,
    SelectChannel,
    EnterPrice {
        channel_id: i64,
        channel_name: String
    },
    EnterCryptoAddress {
        channel_id: i64
    },
    PriceDialogueEnding {
        channel_id: i64,
        crypto_address: String
    }
}

// Define states for the second dialogue (Pay)
#[derive(Clone, Default)]
pub enum PayState {
    #[default]
    Start,
    SelectChannel,
    Pay {
        channel_id: i64,
        channel_name: String
    },
    PaymentStatus
}