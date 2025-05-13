// Define states for the second dialogue (SetPrice)
#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize)]
pub enum State {
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