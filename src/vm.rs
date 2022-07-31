use std::collections::HashMap;
use std::fmt::Display;

use sqlparser::parser::ParserError;

use crate::codegen::{codegen, CodegenError};
use crate::column::Column;
use crate::ic::{Instruction, IntermediateCode};
use crate::parser::parse;
use crate::table::{Row, Table};
use crate::value::Value;
use crate::{BoundedString, Database, Mrc};

const DEFAULT_DATABASE_NAME: &str = "default";

/// An index that can be used to access a specific register.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct RegisterIndex(usize);

impl RegisterIndex {
    /// Get the next index in the sequence.
    pub fn next_index(&self) -> RegisterIndex {
        RegisterIndex(self.0 + 1)
    }
}

impl Display for RegisterIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "%{}", self.0)
    }
}

/// An index that can be used as a reference to a table.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TableIndex(usize);

impl TableIndex {
    /// Get the next index in the sequence.
    pub fn next_index(&self) -> Self {
        TableIndex(self.0 + 1)
    }
}

/// Executor of an SQL query.
pub struct VirtualMachine {
    database: Database,
    registers: HashMap<RegisterIndex, Register>,
    tables: HashMap<TableIndex, Table>,
    last_table_index: TableIndex,
}

impl VirtualMachine {
    pub fn new(name: BoundedString) -> Self {
        Self {
            database: Database::new(name),
            registers: Default::default(),
            tables: Default::default(),
            last_table_index: Default::default(),
        }
    }

    /// Inserts a value for the register at the given index.
    pub fn insert_register(&mut self, index: RegisterIndex, reg: Register) {
        self.registers.insert(index.clone(), reg);
    }

    /// Gets the value for the register at the given index.
    pub fn get_register(&mut self, index: &RegisterIndex) -> Option<&Register> {
        self.registers.get(index)
    }

    /// Creates a new table with a temp name and returns its index.
    pub fn new_temp_table(&mut self) -> TableIndex {
        let index = self.last_table_index.next_index();
        self.tables.insert(index, Table::new_temp(index.0));
        index
    }

    /// Get a reference to an existing table at the given index.
    pub fn table(&self, index: &TableIndex) -> Option<&Table> {
        self.tables.get(index)
    }

    /// Drop an existing table from the VM.
    ///
    /// Note: does NOT remove the table from the schema (if it was added to a schema).
    // TODO: ensure that IC gen calls this when a temp table is created.
    pub fn drop_table(&mut self, index: &TableIndex) {
        self.tables.remove(index);
    }

    /// Executes the given SQL.
    // TODO: fix return type
    pub fn execute(&mut self, code: &str) -> Result<(), ExecutionError> {
        let ast = parse(code)?;
        for stmt in ast {
            let ic = codegen(&stmt)?;
            self.execute_ic(&ic)?;
        }
        Ok(())
    }

    /// Executes the given intermediate code.
    // TODO: fix return type
    fn execute_ic(&mut self, ic: &IntermediateCode) -> Result<(), ExecutionError> {
        for instr in &ic.instrs {
            self.execute_instr(instr)?;
        }
        Ok(())
    }

    /// Executes the given instruction.
    // TODO: fix return type
    fn execute_instr(&mut self, _instr: &Instruction) -> Result<(), ExecutionError> {
        let _ = &self.database;
        todo!()
    }
}

impl Default for VirtualMachine {
    fn default() -> Self {
        Self::new(DEFAULT_DATABASE_NAME.into())
    }
}

/// A register in the executor VM.
pub enum Register {
    /// A reference to a table.
    TableRef(TableIndex),
    /// A grouped table.
    GroupedTable {
        grouped_col: Column,
        other_cols: Vec<Column>,
        /// The group, a mapping of grouped col value -> rows in that group.
        data: Vec<(Value, Vec<Row>)>,
    },
    /// A table definition.
    TableDef(TableDef),
    /// A column definition
    Column(Column),
    /// An insert statement
    InsertDef(InsertDef),
    /// A row to insert
    InsertRow(InsertRow),
    // TODO: an error value?
}

/// An abstract definition of a create table statement.
pub struct TableDef {
    pub name: BoundedString,
    pub columns: Vec<Column>,
}

/// An abstract definition of an insert statement.
pub struct InsertDef {
    /// The view to insert into
    pub table_name: BoundedString,
    /// The columns to insert into.
    ///
    /// Empty means all columns.
    pub columns: Vec<Column>,
    /// The values to insert.
    pub rows: Vec<InsertRow>,
}

impl InsertDef {
    pub fn new(table_name: BoundedString) -> Self {
        Self {
            table_name,
            columns: Vec::new(),
            rows: Vec::new(),
        }
    }
}

/// A row of values to insert.
pub struct InsertRow {
    /// The values
    pub values: Vec<Value>,
    /// The insert definition which this belongs to
    pub def: Mrc<InsertDef>,
}

pub enum ExecutionError {
    ParseError(ParserError),
    CodegenError(CodegenError),
}

impl From<ParserError> for ExecutionError {
    fn from(err: ParserError) -> Self {
        ExecutionError::ParseError(err)
    }
}

impl From<CodegenError> for ExecutionError {
    fn from(err: CodegenError) -> Self {
        ExecutionError::CodegenError(err)
    }
}

#[cfg(test)]
mod tests {
    use super::VirtualMachine;

    #[test]
    fn create_vm() {
        let _ = VirtualMachine::default();
    }
}
