use crate::{u256_ext::ToBigDecimal, web3::types::Address, U256};
use std::fmt;

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Erc20Quantity {
    token_contract: Address,
    quantity: U256,
}

impl fmt::Display for Erc20Quantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.quantity.to_bigdec(0))
    }
}

impl Erc20Quantity {
    pub fn new(token_contract: Address, quantity: U256) -> Self {
        Erc20Quantity {
            token_contract,
            quantity,
        }
    }

    pub fn token_contract(&self) -> Address {
        self.token_contract
    }

    pub fn quantity(&self) -> U256 {
        self.quantity
    }
}

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Erc20Token {
    pub symbol: String,
    pub decimals: u16,
    pub address: Address,
}
