// Define states for the second dialogue (SetPrice)
#[derive(Clone, Default)]
pub enum PriceState {
    #[default]
    Start,
    SelectChannel,
    EnterPrice {
        channel_id: i64,
        channel_name: String
    }
}
