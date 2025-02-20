//   Copyright 2022. The Tari Project
//
//   Redistribution and use in source and binary forms, with or without modification, are permitted provided that the
//   following conditions are met:
//
//   1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following
//   disclaimer.
//
//   2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the
//   following disclaimer in the documentation and/or other materials provided with the distribution.
//
//   3. Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote
//   products derived from this software without specific prior written permission.
//
//   THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES,
//   INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
//   DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
//   SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
//   SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY,
//   WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE
//   USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
use tari_template_lib::prelude::*;
use tari_template_lib::rand::{random_bytes, random_u32};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DSizeMeasure {
    pub brightness: u32,
}

struct WeightedRandomArgs {
    amount: Amount,
    max_amount: f64,
    max: f64,
    min: f64,
}

#[template]
mod d_size_measure {
    use super::*;

    pub struct DSizeMeasureNft {
        counter: u32,
        resource_address: ResourceAddress,
        vault: Vault,
    }

    impl DSizeMeasureNft {
        pub fn new(initial_supply: Amount, token_symbol: String) -> Self {
            let coins = ResourceBuilder::fungible()
                .with_token_symbol(&token_symbol)
                .initial_supply(initial_supply);

            let resource_address = ResourceBuilder::non_fungible()
                .with_token_symbol("D-SIZE-MEASURE")
                .with_access_rules(ResourceAccessRules::new().mintable(AccessRule::AllowAll))
                .with_owner_rule(OwnerRule::ByAccessRule(AccessRule::AllowAll))
                .build();

            Self {
                resource_address,
                vault: Vault::from_bucket(coins),
                counter: 0,
            }
        }

        fn weighted_random(&mut self, params: WeightedRandomArgs) -> f64 {
            let WeightedRandomArgs {
                amount,
                max_amount,
                max,
                min,
            } = params;
            
            // Get random bytes and convert to f64 between 0 and 1
            let random_bytes = random_bytes(8);
            let random_value = u64::from_le_bytes(random_bytes.try_into().unwrap()) as f64;
            let rand_normalized = random_value / u64::MAX as f64;

            let mut value = amount.as_u64_checked().unwrap() as f64;

            if value > max_amount {
                value = max_amount;
            }

            // Normalize input to control weighting (ensures value is between 0 and 1)
            let weight = (value / 100.0).clamp(0.0, 1.0);

            // Generate a biased random number using exponential weighting
            let rand_factor = rand_normalized.powf(1.0 - weight);
            let result = min + rand_factor * (max - min);

            result
        }

        pub fn mint(&mut self, burnAmount: Amount) -> Bucket {
            debug!(format!("Minting {} tokens", burnAmount));
            // These are characteristic of the NFT and are immutable
            let mut immutable_data = Metadata::new();

            let length = self.weighted_random(WeightedRandomArgs {
                amount: burnAmount,
                max_amount: 90.0,
                max: 10.0,
                min: 0.2,
            });

            // Use random_u32 for generating random numbers
            let shiny = random_u32() % 100 < 1;
            let hardness = match random_u32() % 100 {
                n if n <= 50 => "soft",
                n if n <= 70 => "firm",
                n if n <= 90 => "hard",
                n if n <= 98 => "rigid",
                _ => "unyielding",
            };

            immutable_data
                .insert("name", format!("D-SIZE #{}", self.counter))
                .insert("hardness", hardness.to_string())
                .insert("length", length.to_string())
                .insert("shiny", shiny.to_string())
                .insert("number", self.counter.to_string())
                .insert("burn_amount", burnAmount.to_string());

            self.counter += 1;

            let bucket = self.vault.withdraw(burnAmount);
            bucket.burn();

            let res_manager = ResourceManager::get(self.resource_address);
            res_manager.mint_non_fungible(
                NonFungibleId::from_u32(self.counter - 1),
                &immutable_data,
                &DSizeMeasure { brightness: 0 },
            )
        }

        pub fn vault_address(&self) -> ResourceAddress {
            self.vault.resource_address()
        }

        pub fn total_supply(&self) -> Amount {
            ResourceManager::get(self.resource_address).total_supply()
        }

        pub fn burn(&mut self, bucket: Bucket) {
            assert!(
                bucket.resource_type() == ResourceType::NonFungible,
                "The resource is not a NFT"
            );
            assert!(
                bucket.resource_address() == self.resource_address,
                "Cannot burn bucket not from this collection"
            );
            debug!(format!(
                "Burning bucket {} containing {}",
                bucket.id(),
                bucket.amount()
            ));
            bucket.burn();
        }
    }
}
