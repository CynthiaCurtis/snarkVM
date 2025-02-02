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

mod input;
use input::*;

mod bytes;
mod parse;

use console::{
    network::prelude::*,
    program::{Identifier, PlaintextType, Register},
};

use indexmap::IndexSet;
use std::collections::HashMap;

pub trait FinalizeCommandTrait: Clone + PartialEq + Eq + Parser + FromBytes + ToBytes {
    /// Returns the number of operands.
    fn num_operands(&self) -> usize;
}

pub trait CommandTrait<N: Network>: Clone + Parser + FromBytes + ToBytes {
    type FinalizeCommand: FinalizeCommandTrait;

    /// Returns the destination registers of the command.
    fn destinations(&self) -> Vec<Register<N>>;
    /// Returns the branch target, if the command is a branch command.
    fn branch_to(&self) -> Option<&Identifier<N>>;
    /// Returns the position name, if the command is a position command.
    fn position(&self) -> Option<&Identifier<N>>;
    /// Returns `true` if the command is a call instruction.
    fn is_call(&self) -> bool;
    /// Returns `true` if the command is a cast to record instruction.
    fn is_cast_to_record(&self) -> bool;
    /// Returns `true` if the command is a write operation.
    fn is_write(&self) -> bool;
}

#[derive(Clone, PartialEq, Eq)]
pub struct FinalizeCore<N: Network, Command: CommandTrait<N>> {
    /// The name of the associated function.
    name: Identifier<N>,
    /// The input statements, added in order of the input registers.
    /// Input assignments are ensured to match the ordering of the input statements.
    inputs: IndexSet<Input<N>>,
    /// The commands, in order of execution.
    commands: Vec<Command>,
    /// The number of write commands.
    num_writes: u16,
    /// A mapping from `Position`s to their index in `commands`.
    positions: HashMap<Identifier<N>, usize>,
}

impl<N: Network, Command: CommandTrait<N>> FinalizeCore<N, Command> {
    /// Initializes a new finalize with the given name.
    pub fn new(name: Identifier<N>) -> Self {
        Self { name, inputs: IndexSet::new(), commands: Vec::new(), num_writes: 0, positions: HashMap::new() }
    }

    /// Returns the name of the associated function.
    pub const fn name(&self) -> &Identifier<N> {
        &self.name
    }

    /// Returns the finalize inputs.
    pub const fn inputs(&self) -> &IndexSet<Input<N>> {
        &self.inputs
    }

    /// Returns the finalize input types.
    pub fn input_types(&self) -> Vec<PlaintextType<N>> {
        self.inputs.iter().map(|input| *input.plaintext_type()).collect()
    }

    /// Returns the finalize commands.
    pub fn commands(&self) -> &[Command] {
        &self.commands
    }

    /// Returns the number of write commands.
    pub const fn num_writes(&self) -> u16 {
        self.num_writes
    }

    /// Returns the mapping of `Position`s to their index in `commands`.
    pub const fn positions(&self) -> &HashMap<Identifier<N>, usize> {
        &self.positions
    }
}

impl<N: Network, Command: CommandTrait<N>> FinalizeCore<N, Command> {
    /// Adds the input statement to finalize.
    ///
    /// # Errors
    /// This method will halt if a command was previously added.
    /// This method will halt if the maximum number of inputs has been reached.
    /// This method will halt if the input statement was previously added.
    #[inline]
    fn add_input(&mut self, input: Input<N>) -> Result<()> {
        // Ensure there are no commands in memory.
        ensure!(self.commands.is_empty(), "Cannot add inputs after commands have been added");

        // Ensure the maximum number of inputs has not been exceeded.
        ensure!(self.inputs.len() <= N::MAX_INPUTS, "Cannot add more than {} inputs", N::MAX_INPUTS);
        // Ensure the input statement was not previously added.
        ensure!(!self.inputs.contains(&input), "Cannot add duplicate input statement");

        // Ensure the input register is a locator.
        ensure!(matches!(input.register(), Register::Locator(..)), "Input register must be a locator");

        // Insert the input statement.
        self.inputs.insert(input);
        Ok(())
    }

    /// Adds the given command to finalize.
    ///
    /// # Errors
    /// This method will halt if the maximum number of commands has been reached.
    #[inline]
    pub fn add_command(&mut self, command: Command) -> Result<()> {
        // Ensure the maximum number of commands has not been exceeded.
        ensure!(self.commands.len() < N::MAX_COMMANDS, "Cannot add more than {} commands", N::MAX_COMMANDS);
        // Ensure the number of write commands has not been exceeded.
        ensure!(self.num_writes < N::MAX_WRITES, "Cannot add more than {} 'set' commands", N::MAX_WRITES);

        // Ensure the command is not a call instruction.
        ensure!(!command.is_call(), "Forbidden operation: Finalize cannot invoke a 'call'");
        // Ensure the command is not a cast to record instruction.
        ensure!(!command.is_cast_to_record(), "Forbidden operation: Finalize cannot cast to a record");

        // Check the destination registers.
        for register in command.destinations() {
            // Ensure the destination register is a locator.
            ensure!(matches!(register, Register::Locator(..)), "Destination register must be a locator");
        }

        // Check if the command is a branch command.
        if let Some(position) = command.branch_to() {
            // Ensure the branch target does not reference an earlier position.
            ensure!(!self.positions.contains_key(position), "Cannot branch to an earlier position '{position}'");
        }

        // Check if the command is a position command.
        if let Some(position) = command.position() {
            // Ensure the position is not yet defined.
            ensure!(!self.positions.contains_key(position), "Cannot redefine position '{position}'");
            // Ensure that there are less than `u8::MAX` positions.
            ensure!(self.positions.len() < u8::MAX as usize, "Cannot add more than {} positions", u8::MAX);
            // Insert the position.
            self.positions.insert(*position, self.commands.len());
        }

        // Check if the command is a write command.
        if command.is_write() {
            // Increment the number of write commands.
            self.num_writes += 1;
        }

        // Insert the command.
        self.commands.push(command);
        Ok(())
    }
}

impl<N: Network, Command: CommandTrait<N>> TypeName for FinalizeCore<N, Command> {
    /// Returns the type name as a string.
    #[inline]
    fn type_name() -> &'static str {
        "finalize"
    }
}
