use crate::core::btree::BTree;
use crate::parser::lexer::Lexer;
use crate::parser::parser::{
    CreateTableQuery, DeleteQuery, Expr, InsertQuery, Parser, Query, SelectQuery, WhereClause,
};
use std::collections::{HashMap, HashSet};
use std::i64;
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

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub enum DataType {
    Int(i64),
    Text(String),
}

impl Default for DataType {
    fn default() -> Self {
        DataType::Int(0)
    }
}

impl DataType {
    pub fn get_value(&self) -> &dyn std::fmt::Display {
        match self {
            DataType::Int(val) => val,
            DataType::Text(val) => val,
        }
    }
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

    pub fn get_column(&self, name: &str) -> Option<&Column> {
        self.columns.iter().find(|col| col.name == name)
    }

    pub fn validate_column(schema_col: &Column, val: &Expr) -> Option<DataType> {
        match schema_col.column_type {
            ColumnType::String => match val {
                Expr::Ident(ref s) => Some(DataType::Text(s.to_string())),
                _ => None,
            },
            ColumnType::Int => match val {
                Expr::Number(n) => Some(DataType::Int(*n)),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn validate_insert_row(
        &self,
        columns: &Vec<String>,
        values: &Vec<Expr>,
    ) -> Result<Vec<DataType>, String> {
        let mut result: Vec<DataType> = vec![];
        for (i, col) in columns.iter().enumerate() {
            match self.get_column(&col) {
                Some(schema_col) => match Self::validate_column(schema_col, &values[i]) {
                    Some(datatype) => result.push(datatype),
                    None => {
                        return Err(format!(
                            "Invalid type for column {} expected {:?} got {:?}",
                            col, schema_col.column_type, values[i]
                        ));
                    }
                },
                None => return Err(format!("Did you just create the column: {}", &col)),
            }
        }
        Ok(result)
    }

    pub fn validate_row(&self, values: &Vec<String>) -> Result<(), String> {
        for col in &self.columns {
            if let Some(value) = values.get((col.order - 1) as usize) {
                match col.column_type {
                    ColumnType::Int => {
                        value.parse::<i64>().expect("invalid int value");
                    }
                    ColumnType::String => (),
                    ColumnType::Date => (),
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
    rows: BTree<DataType, HashMap<String, DataType>>,
}

impl Table {
    pub fn new() -> Self {
        Self {
            schema: Schema::new(),
            rows: BTree::new(2),
        }
    }

    pub fn insert_row(&mut self, columns: Vec<String>, values_: Vec<Expr>) -> Result<(), String> {
        let values = self.schema.validate_insert_row(&columns, &values_)?;
        let id = columns
            .iter()
            .position(|s| s == "id")
            .ok_or_else(|| "expected an 'id' column".to_string())?;
        /*
        let id = row
            .get("id")
            .ok_or_else(|| "missing primary key".to_string())?
            .clone();
        */
        if self.rows.search(&values[id]).is_some() {
            return Err("Duplicate primary key".to_string());
        }

        let mut rows = HashMap::new();
        for i in 0..columns.len() {
            rows.insert(columns[i].clone(), values[i].clone());
        }
        self.rows.insert(values[id].clone(), rows);
        Ok(())
    }

    /*
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
    */

    fn delete_row(&mut self, query: DeleteQuery) -> Result<(), String> {
        if let Some(c) = query.where_clause {
            match c.left {
                Expr::Ident(ref s) if s != "id" => {
                    return Err("Buddy you have high expecations".to_string());
                }
                _ => {}
            }
            let right_value = match c.right {
                Expr::Number(ref s) => Some(DataType::Int(*s)),
                _ => return Err("man cmon , dont you know the id is a number??????".to_string()),
            }
            .expect("expected a valid value in right");

            self.rows.delete(&right_value);
        } else {
            return Err(
                "Expected a where clause , do you want me to delete the whole table??????"
                    .to_string(),
            );
        }
        Ok(())
    }

    fn get_row_from_where_clause(
        &self,
        clause: &Option<WhereClause>,
    ) -> Result<HashMap<String, DataType>, String> {
        if let Some(c) = clause {
            let where_clause = c.clone();
            match &where_clause.left {
                Expr::Ident(ref s) if s != "id" => {
                    return Err("Buddy you have high expecations".to_string());
                }
                _ => {}
            };

            let right_value = match where_clause.right {
                Expr::Ident(ref s) => Some(DataType::Text(s.to_string())),
                Expr::Number(ref s) => Some(DataType::Int(*s)),
            }
            .expect("expected a valid value in right");
            let row = self.rows.search(&right_value);
            Ok(row.ok_or_else(|| "Couldnt find a row").cloned()?)
        } else {
            Err("No Where Clause".to_string())
        }
    }

    pub fn select_rows(&self, query: &SelectQuery) -> Option<Vec<HashMap<String, DataType>>> {
        if let Ok(row) = self.get_row_from_where_clause(&query.where_clause) {
            let result: HashMap<String, DataType> = {
                if query.columns[0] != "*" {
                    query
                        .columns
                        .iter()
                        .filter_map(|s| row.get(s).map(|v| (s.clone(), v.clone())))
                        .collect()
                } else {
                    row
                }
            };
            return Some(vec![result]);
        } else {
            if query.where_clause.is_none() {
                // TODO:this is really bad , should be refactored

                let result: Vec<HashMap<String, DataType>> = {
                    if query.columns[0] == "*" {
                        self.rows.values_in_order()
                    } else {
                        self.rows
                            .values_in_order()
                            .iter()
                            .map(|row| {
                                query
                                    .columns
                                    .iter()
                                    .filter_map(|s| row.get(s).map(|v| (s.clone(), v.clone())))
                                    .collect()
                            })
                            .collect()
                    }
                };
                return Some(result);
            } else {
                None
            }
        }
    }
}

#[derive(Debug)]
pub struct Neoqlite {
    tables: HashMap<String, Table>,
    debug: bool,
}

impl Neoqlite {
    pub fn new() -> Self {
        let mut tables = HashMap::new();
        let mut user_table = Table::new();
        user_table
            .schema
            .add_column("id".to_string(), ColumnType::Int);
        user_table
            .schema
            .add_column("username".to_string(), ColumnType::String);
        user_table
            .schema
            .add_column("email".to_string(), ColumnType::String);

        user_table
            .schema
            .add_column("otp".to_string(), ColumnType::Int);

        let mut dummy_table = Table::new();
        dummy_table
            .schema
            .add_column("id".to_string(), ColumnType::Int);
        dummy_table
            .schema
            .add_column("username".to_string(), ColumnType::String);
        dummy_table
            .schema
            .add_column("email".to_string(), ColumnType::String);

        tables.insert("users".to_string(), user_table);
        tables.insert("dummy".to_string(), dummy_table);

        let debug = false;
        Self { tables, debug }
    }

    //  GOOD OLD OOP HUH
    pub fn set_debug(&mut self, value: bool) {
        self.debug = value;
    }

    pub fn exec_meta_command(&self, input: &str) {
        match input {
            ".exit" => exit(0),
            ".bt" => {
                for (table_name, table) in &self.tables {
                    println!("Table : {}", table_name);
                    println!("\n{:?}\n", table);
                }
            }
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
        table.insert_row(query.columns, query.values)
    }

    pub fn exec_select(&self, query: SelectQuery) -> Result<(), String> {
        let table = self
            .tables
            .get(&query.table)
            .ok_or_else(|| "Table not found".to_string())?;

        let result = table.select_rows(&query).ok_or("None")?;
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
        table.delete_row(query)?;
        Ok(())
    }

    pub fn exec_create_table(&mut self, query: CreateTableQuery) -> Result<(), String> {
        if self.tables.get(&query.table).is_some() {
            return Err(format!("A Table with name '{}' exists", query.table));
        }

        let mut did_have_id = false;

        let mut new_table = Table::new();

        for (col, col_type) in &query.columns {
            if col == "id" {
                if *col_type != ColumnType::Int {
                    return Err(format!("Expected 'id' to be an int"));
                } else {
                    did_have_id = true;
                }
            }

            new_table.schema.add_column(col.clone(), col_type.clone());
        }

        if !did_have_id {
            return Err(format!("Expected an 'id' column"));
        }

        self.tables.insert(query.table, new_table);
        Ok(())
    }

    pub fn exec(&mut self, query: Query) -> Result<(), String> {
        if self.debug {
            println!("\n\nParserResult:\n{:?}\n\n", query)
        };
        match query {
            Query::Insert(query) => self.exec_insert(query)?,
            Query::Select(query) => self.exec_select(query)?,
            Query::Delete(query) => self.exec_delete(query)?,
            Query::CreateTable(query) => self.exec_create_table(query)?,
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
