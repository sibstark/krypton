// Define states for the second dialogue (SetPrice)
#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub enum PriceState {
    #[default]
    Start,
    PriceSelectChannel,
    EnterPrice {
        channel_id: i64,
        channel_name: String
    },
    EnterCryptoAddress {
        channel_id: i64
    }
}

// Define states for the second dialogue (Pay)
#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub enum PayState {
    #[default]
    Start,
    SelectChannel {
        channel_id: Option<i64>
    },
    Pay {
        channel_id: i64,
        channel_name: String
    },
    PaymentStatus
}

// Define states for the second dialogue (ShowInfoState)
#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub enum ShowInfoState {
    #[default]
    Start,
    SelectChannel
}