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

use snarkvm_circuit_network::Aleo;
use snarkvm_circuit_types::{environment::prelude::*, Boolean, Field, U8};

#[derive(Clone)]
pub struct HeaderLeaf<A: Aleo> {
    /// The index of the Merkle leaf.
    index: U8<A>,
    /// The ID.
    id: Field<A>,
}

impl<A: Aleo> HeaderLeaf<A> {
    /// Returns the index of the Merkle leaf.
    pub fn index(&self) -> &U8<A> {
        &self.index
    }

    /// Returns the ID in the Merkle leaf.
    pub const fn id(&self) -> &Field<A> {
        &self.id
    }
}

impl<A: Aleo> Inject for HeaderLeaf<A> {
    type Primitive = console::HeaderLeaf<A::Network>;

    /// Initializes a new header leaf circuit from a primitive.
    fn new(mode: Mode, leaf: Self::Primitive) -> Self {
        Self { index: U8::new(mode, console::U8::new(leaf.index())), id: Field::new(mode, leaf.id()) }
    }
}

impl<A: Aleo> Eject for HeaderLeaf<A> {
    type Primitive = console::HeaderLeaf<A::Network>;

    /// Ejects the mode of the header leaf.
    fn eject_mode(&self) -> Mode {
        (&self.index, &self.id).eject_mode()
    }

    /// Ejects the header leaf.
    fn eject_value(&self) -> Self::Primitive {
        Self::Primitive::new(*self.index.eject_value(), self.id.eject_value())
    }
}

impl<A: Aleo> ToBits for HeaderLeaf<A> {
    type Boolean = Boolean<A>;

    /// Outputs the little-endian bit representation of `self` *without* trailing zeros.
    fn to_bits_le(&self) -> Vec<Self::Boolean> {
        let mut bits_le = self.index.to_bits_le();
        bits_le.extend(self.id.to_bits_le());
        bits_le
    }

    /// Outputs the big-endian bit representation of `self` *without* leading zeros.
    fn to_bits_be(&self) -> Vec<Self::Boolean> {
        let mut bits_be = self.index.to_bits_be();
        bits_be.extend(self.id.to_bits_be());
        bits_be
    }
}
