use std::convert::TryInto;
use std::fmt;

use lightning::ln::PaymentHash;
use rgbstd::contract::ContractId;

use crate::utils::hex_str_to_vec;

#[derive(Debug, Clone, Copy)]
pub enum SwapType {
    BuyAsset { asset_amount: u64, amt_msat: u64 },
    SellAsset { asset_amount: u64, amt_msat: u64 },
}

impl fmt::Display for SwapType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SwapType::BuyAsset { .. } => write!(f, "buy"),
            SwapType::SellAsset { .. } => write!(f, "sell"),
        }
    }
}

impl SwapType {
    pub fn opposite(self) -> Self {
        match self {
            SwapType::BuyAsset {
                asset_amount,
                amt_msat,
            } => SwapType::SellAsset {
                asset_amount,
                amt_msat,
            },
            SwapType::SellAsset {
                asset_amount,
                amt_msat,
            } => SwapType::BuyAsset {
                asset_amount,
                amt_msat,
            },
        }
    }

    pub fn is_buy(&self) -> bool {
        matches!(self, SwapType::BuyAsset { .. })
    }

    pub fn amt_msat(&self) -> u64 {
        match self {
            SwapType::BuyAsset { amt_msat, .. } | SwapType::SellAsset { amt_msat, .. } => *amt_msat,
        }
    }
    pub fn asset_amount(&self) -> u64 {
        match self {
            SwapType::BuyAsset { asset_amount, .. } | SwapType::SellAsset { asset_amount, .. } => {
                *asset_amount
            }
        }
    }
}

#[derive(Debug)]
pub struct SwapString {
    pub contract_id: ContractId,
    pub swap_type: SwapType,
    pub expiry: u64,
    pub payment_hash: PaymentHash,
}

impl std::str::FromStr for SwapString {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.split('/');
        let amount = iter.next();
        let contract_id = iter.next();
        let side = iter.next();
        let price = iter.next();
        let expiry = iter.next();
        let payment_hash = iter.next();

        if payment_hash.is_none() || iter.next().is_some() {
            return Err("Wrong number of parts");
        }

        let amount = amount.unwrap().parse::<u64>();
        let contract_id = ContractId::from_str(contract_id.unwrap());
        let price = price.unwrap().parse::<u64>();
        let expiry = expiry.unwrap().parse::<u64>();
        let payment_hash = hex_str_to_vec(payment_hash.unwrap())
            .and_then(|vec| vec.try_into().ok())
            .map(PaymentHash);

        if amount.is_err()
            || contract_id.is_err()
            || price.is_err()
            || expiry.is_err()
            || payment_hash.is_none()
        {
            return Err("Unable to parse");
        }

        let amount = amount.unwrap();
        let contract_id = contract_id.unwrap();
        let price = price.unwrap();
        let expiry = expiry.unwrap();
        let payment_hash = payment_hash.unwrap();

        if amount == 0 || price == 0 || expiry == 0 {
            return Err("Amount, price and expiry should be positive");
        }

        let amt_msat = amount * price;
        let swap_type = match side {
            Some("buy") => SwapType::BuyAsset {
                asset_amount: amount,
                amt_msat,
            },
            Some("sell") => SwapType::SellAsset {
                asset_amount: amount,
                amt_msat,
            },
            _ => {
                return Err("Invalid swap type");
            }
        };

        Ok(SwapString {
            contract_id,
            swap_type,
            expiry,
            payment_hash,
        })
    }
}
