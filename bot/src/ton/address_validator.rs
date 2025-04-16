use ton_address::Address;


pub fn is_valid_address(address: &str) -> bool {
    match Address::from_base64(address, None) {
        Ok(_) => true,
        Err(_) => false,
    }
}