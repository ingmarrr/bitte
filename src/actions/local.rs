use std::path::PathBuf;

use rusqlite::params;

use crate::{consts, err::DbErr, Template};

pub struct Local {
    con: rusqlite::Connection,
}

impl Local {
    pub fn new() -> Result<Self, DbErr> {
        let home_dir = std::env::var("HOME").map_err(|_| DbErr::HomeNotFound)?;
        let db_path = PathBuf::from(home_dir).join(consts::DB_PATH);
        println!("DB Path: {:?}", db_path);
        std::fs::create_dir_all(db_path.parent().unwrap())?;
        let con = rusqlite::Connection::open(db_path)?;
        let local = Self { con };
        local.init()?;
        Ok(local)
    }

    pub fn add(&self, template: Template) -> Result<(), DbErr> {
        let res = self.con.execute(
            "INSERT INTO templates (name, content) VALUES (?1, ?2)",
            params![template.name, template.body],
        )?;
        println!("Rows changed: {:#?}", res);
        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<Template, DbErr> {
        Ok(self.con.query_row(
            "SELECT name, content FROM templates WHERE name = ?1",
            params![key],
            |row| {
                Ok(Template {
                    name: row.get(0)?,
                    body: row.get(1)?,
                })
            },
        )?)
    }

    pub fn update(&self, template: Template) -> Result<(), DbErr> {
        let res = self.con.execute(
            "UPDATE templates SET content = ?1 WHERE name = ?2",
            params![template.body, template.name],
        )?;
        println!("Rows changed: {:#?}", res);
        Ok(())
    }

    pub fn upsert(&self, template: Template) -> Result<(), DbErr> {
        let res = self.con.execute(
            "INSERT INTO templates (name, content) VALUES (?1, ?2) ON CONFLICT(name) DO UPDATE SET content = ?2",
            params![template.name, template.body],
        )?;
        println!("Rows changed: {:#?}", res);
        Ok(())
    }

    pub fn del(&self, key: &str) -> Result<(), DbErr> {
        let res = self
            .con
            .execute("DELETE FROM templates WHERE name = ?1", params![key])?;
        println!("Rows changed: {:#?}", res);
        Ok(())
    }

    #[rustfmt::skip]
    fn init(&self) -> Result<(), rusqlite::Error> {
        let _ = self.con.execute(
            "CREATE TABLE IF NOT EXISTS templates (
            name TEXT PRIMARY KEY,
            content TEXT NOT NULL
        )", params![],)?;
        Ok(())
    }
}
