# Neoqlite  

Neoqlite is a lightweight, in-memory SQL database written in **Rust**. It aims to mimic (or to be like) the core functionality and power of **SQLite** while providing a simple interface for query execution and database management.

This project is in its **early stages** 

---

## Features  

- **SQL-like Interface**: Supports basic SQL commands including `CREATE`, `INSERT`, `SELECT`, `DELETE`.  
- **In-Memory Storage**: All data is stored in memory for fast execution (persistent storage planned in the future).  
- **Interactive Shell**: Includes an interactive command-line interface for executing SQL statements dynamically.  
- **Debug Mode**: View executed queries and debug output.  

---

## Requirements  

- **Rust**: Ensure the latest version of Rust is installed. Install it via [rustup](https://rustup.rs/).  
- **Tokio**: Used for asynchronous support. Managed automatically by `Cargo.toml`.  

---

## Installation  

Clone the repository and navigate to the project folder:

```bash
git clone https://github.com/your-username/neoqlite.git
cd neoqlite
```

Build the project:

```bash
cargo build
```

Run the project:

```bash
cargo run
```

---

## Usage  

Once running, Neoqlite starts an **interactive shell** where you can execute SQL statements:

### Example Commands  

1. **Create a Table**  

```sql
CREATE TABLE users (
    id INT,
    email TEXT NOTNULL,
    username TEXT
);
```

2. **Insert Data**  

```sql
INSERT INTO users (id, email, username) VALUES (1, 'masoom@email.com', 'masoom');
INSERT INTO users (id, email, username) VALUES (2, 'nani@email.com', 'nani');
```

3. **Select Data**  

```sql
SELECT * FROM users;
```

4. **Delete Data**  

```sql
DELETE FROM users WHERE id = 2;
```

5. **Exit**  

Type `.q` to quit the interactive shell.  

---

## Code Walkthrough  

### Key Components  

1. **Database Initialization**  
   Neoqlite initializes an in-memory database and allows SQL commands to be executed using `exec_stmt`.

2. **Debug Mode**  
   Enable debug mode to print executed statements and outputs:  
   ```rust
   neoqlite.set_debug(true);
   ```

3. **Interactive Input**  
   The program reads input dynamically using a simple loop and executes it against the in-memory database.  

---

## Future Roadmap  

- Persistent storage (file-based database).  
- Enhanced SQL support: Joins, transactions, and indexes.  
- CLI improvements for better user interaction.  
- Error handling improvements.  

---

## Contribution  

Contributions are welcome! To contribute:  

1. Fork the repository.  
2. Create a new branch:  
   ```bash
   git checkout -b feature/your-feature
   ```
3. Commit and push your changes.  
4. Open a Pull Request.  

---
