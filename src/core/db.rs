use crate::core::btree::BTree;
use crate::parser::lexer::Lexer;
use crate::parser::parser::{
    DeleteQuery, Expr, InsertQuery, Parser, Query, SelectQuery, WhereClause,
};
use std::collections::{HashMap, HashSet};
use std::process::exit;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum ColumnType {
    Int,
    String,
    Date,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Constraints {
    Unique,
    NotNull,
    Null,
    PrimaryKey,
    ForeignKey,
}

#[derive(Debug, Hash, Eq, PartialEq)]
pub struct Column {
    name: String,
    column_type: ColumnType,
    constraints: Vec<Constraints>,
    order: u8,
}

impl Column {
    fn new(name: &str, column_type: ColumnType, constraints: &[Constraints], order: u8) -> Self {
        Self {
            name: name.to_string(),
            column_type,
            constraints: constraints.to_vec(),
            order,
        }
    }
}

#[derive(Debug)]
pub struct Schema {
    columns: HashSet<Column>,
    curr_order: u8,
}

#[derive(Debug, Clone)]
pub enum DataType {
    Int(i64),
    Float(f64),
    Text(String),
}

impl Schema {
    pub fn new() -> Self {
        Self {
            columns: HashSet::new(),
            curr_order: 1,
        }
    }

    pub fn add_column(&mut self, name: String, column_type: ColumnType) {
        let new_column = Column::new(&name, column_type, &[], self.curr_order);
        self.columns.insert(new_column);
        self.curr_order += 1;
    }

    pub fn validate_row(&self, row: &HashMap<String, String>) -> Result<(), String> {
        for col in &self.columns {
            if let Some(value) = row.get(&col.name) {
                match col.column_type {
                    ColumnType::Int => {
                        value.parse::<i64>().expect("invalid int value");
                    }
                    ColumnType::String => (), // Always valid
                    ColumnType::Date => (),   // Extend later for date parsing
                }
            } else {
                return Err(format!("Missing column: {}", &col.name));
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Table {
    schema: Schema,
    rows: BTree<String, Vec<DataType>>,
}

impl Table {
    pub fn new() -> Self {
        Self {
            schema: Schema::new(),
            rows: BTree::new(2),
        }
    }

    pub fn insert_row(&mut self, row: HashMap<String, String>) -> Result<(), String> {
        self.schema.validate_row(&row)?;
        let id = row
            .get("id")
            .ok_or_else(|| "missing primary key".to_string())?
            .clone();

        if self.rows.search(&id).is_some() {
            return Err("Duplicate primary key".to_string());
        }

        self.rows.insert(id, row);
        Ok(())
    }

    pub fn delete_row(&mut self, query: &DeleteQuery) -> Result<(), String> {
        if let Some(q) = &query.where_clause {
            let _left_value = match q.left {
                Expr::Ident(ref s) if s != "id" => return Err("expected id".to_string()),
                Expr::Ident(ref s) => s,
                _ => return Err("wtf is this ".to_string()),
            };

            let right_value = match &q.right {
                Expr::Ident(n) => n,
                _ => return Err("expected a number".to_string()),
            };

            println!("{}", right_value);
            println!("{:?}", self.rows.search(right_value));
            self.rows.delete(&right_value);
        }
        Ok(())
    }

    fn get_row_from_where_clause(
        &self,
        clause: &Option<WhereClause>,
    ) -> Option<HashMap<String, String>> {
        if let Some(c) = clause {
            let where_clause = c.clone();
            match &where_clause.left {
                Expr::Ident(ref s) if s != "id" => {
                    println!("Buddy you have high expecations");
                    return None;
                }
                _ => {}
            }

            let right_value = {
                if let Expr::Ident(ref s) = where_clause.right {
                    Some(s)
                } else {
                    None
                }
            }
            .expect("expected a ident my nigga");
            let row = self.rows.search(right_value);
            row.cloned()
        } else {
            None
        }
    }

    pub fn select_rows(&self, query: &SelectQuery) -> Vec<HashMap<String, String>> {
        if let Some(row) = self.get_row_from_where_clause(&query.where_clause) {
            return vec![row];
        } else {
            return self.rows.values_in_order();
        }
    }
}

#[derive(Debug)]
pub struct Neoqlite {
    tables: HashMap<String, Table>,
}

impl Neoqlite {
    pub fn new() -> Self {
        let mut tables = HashMap::new();
        let mut user_table = Table::new();
        user_table
            .schema
            .add_column("id".to_string(), ColumnType::String);
        user_table
            .schema
            .add_column("username".to_string(), ColumnType::String);
        user_table
            .schema
            .add_column("email".to_string(), ColumnType::String);

        tables.insert("users".to_string(), user_table);
        Self { tables }
    }

    pub fn exec_meta_command(&self, input: &str) {
        match input {
            ".exit" => exit(0),
            _ => println!("Unknown Meta Command"),
        }
    }

    pub fn exec_insert(&mut self, query: InsertQuery) -> Result<(), String> {
        let table = self
            .tables
            .get_mut(&query.table)
            .ok_or_else(|| "Table not found".to_string())?;

        if query.columns.len() != query.values.len() {
            return Err("columns and values are not the same length".to_string());
        }

        let mut row: HashMap<String, String> = HashMap::new();

        for i in 0..query.columns.len() {
            row.insert(query.columns[i].clone(), query.values[i].clone());
        }
        table.insert_row(row)
    }

    pub fn exec_select(&self, query: SelectQuery) -> Result<(), String> {
        let table = self
            .tables
            .get(&query.table)
            .ok_or_else(|| "Table not found".to_string())?;

        let result = table.select_rows(&query);
        for row in result {
            println!("{:?}", row);
        }
        Ok(())
    }

    pub fn exec_delete(&mut self, query: DeleteQuery) -> Result<(), String> {
        let table = &mut self
            .tables
            .get_mut(&query.table)
            .ok_or_else(|| "Table not found".to_string())?;

        table.delete_row(&query)?;
        Ok(())
    }

    pub fn exec(&mut self, query: Query) -> Result<(), String> {
        match query {
            Query::Insert(query) => self.exec_insert(query)?,
            Query::Select(query) => self.exec_select(query)?,
            Query::Delete(query) => self.exec_delete(query)?,
        }
        Ok(())
    }

    pub fn exec_stmt(&mut self, input: &str) -> Result<(), String> {
        if input.starts_with(".") {
            self.exec_meta_command(input);
        } else {
            let parser = Parser::new(Lexer::new(input).tokenize()).parse()?;
            self.exec(parser)?;
        }
        Ok(())
    }
}
