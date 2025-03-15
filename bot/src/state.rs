// Define states for the second dialogue (SetPrice)
#[derive(Clone, Default)]
pub enum PriceState {
    #[default]
    Start,
    SearchChannel,
    ReceivePrice {
        price: u32
    }
}
