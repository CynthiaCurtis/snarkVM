// Copyright (C) 2019-2023 Aleo Systems Inc.
// This file is part of the snarkVM library.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at:
// http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::*;

impl<N: Network> FromBytes for Transactions<N> {
    /// Reads the transactions from buffer.
    #[inline]
    fn read_le<R: Read>(mut reader: R) -> IoResult<Self> {
        // Read the version.
        let version = u8::read_le(&mut reader)?;
        // Ensure the version is valid.
        if version != 0 {
            return Err(error("Invalid transactions version"));
        }
        // Read the number of transactions.
        let num_txs: u32 = FromBytes::read_le(&mut reader)?;
        // Read the transactions.
        let transactions = (0..num_txs).map(|_| FromBytes::read_le(&mut reader)).collect::<Result<Vec<_>, _>>()?;
        // Return the transactions.
        Ok(Self::from(&transactions))
    }
}

impl<N: Network> ToBytes for Transactions<N> {
    /// Writes the transactions to a buffer.
    #[inline]
    fn write_le<W: Write>(&self, mut writer: W) -> IoResult<()> {
        // Write the version.
        0u8.write_le(&mut writer)?;
        // Write the number of transactions.
        u32::try_from(self.transactions.len()).map_err(|e| error(e.to_string()))?.write_le(&mut writer)?;
        // Write the transactions.
        self.transactions.values().try_for_each(|transaction| transaction.write_le(&mut writer))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use console::network::Testnet3;

    type CurrentNetwork = Testnet3;

    #[test]
    fn test_bytes() -> Result<()> {
        let mut rng = TestRng::default();

        for expected in [crate::vm::test_helpers::sample_genesis_block(&mut rng).transactions().clone()].into_iter() {
            // Check the byte representation.
            let expected_bytes = expected.to_bytes_le()?;
            assert_eq!(expected, Transactions::read_le(&expected_bytes[..])?);
            assert!(Transactions::<CurrentNetwork>::read_le(&expected_bytes[1..]).is_err());
        }
        Ok(())
    }
}
