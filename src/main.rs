use neoqlite::core::db::Neoqlite;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), String> {
    //let sql_query = "SELECT name, age FROM users WHERE age > 21;";
    //let sql_query = "INSERT INTO users (id,name,email) values('1','masoom','masoom@email.com');";
    let mut neoqlite = Neoqlite::new();

    neoqlite.exec_stmt(
        "CREATE TABLE otp (
        id int,
        otp text,
        is_valid int
    );",
    )?;

    neoqlite
        .exec_stmt("insert into users(id,email,username) values(1,'masoom','masoom@email.com');")?;
    neoqlite.exec_stmt(
        "insert into users(id,email,username) values(2,'notmasoom','notmasoom@email.com');",
    )?;
    neoqlite
        .exec_stmt("insert into users(id,email,username) values(3,'nani','nani@email.com');")?;
    neoqlite
        .exec_stmt("insert into users(id,email,username) values(4,'nani','nani@email.com');")?;
    neoqlite
        .exec_stmt("insert into users(id,email,username) values(5,'nani','nani@email.com');")?;

    neoqlite.exec_stmt(
        "insert into users(id,email,username,otp) values(6,'mehdi','mehdi@email.com',645);",
    )?;

    neoqlite.set_debug(true);

    neoqlite.exec_stmt("delete from users where id = 5;")?;

    neoqlite.exec_stmt("select * from users;")?;

    //neoqlite.exec_stmt("select * from users where id = 5;")?;

    //println!("{:?}", neoqlite);
    loop {
        print!("neoqlite => ");
        let input: String = get_input().await?;
        let res = neoqlite.exec_stmt(&input);
        if !res.is_ok() {
            println!("\n'{}'\n", res.err().unwrap());
        }
        if input == ".q" {
            break;
        }
    }

    //println!("{}", sql_query);
    Ok(())
}

async fn get_input() -> Result<String, String> {
    let mut input_string: String = String::new();
    io::stdout().flush().unwrap();
    match io::stdin().read_line(&mut input_string) {
        Ok(_) => Ok(input_string.trim().to_string()),
        Err(e) => Err(e.to_string()),
    }
}
