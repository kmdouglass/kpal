use std::error::Error;
use std::fmt;

use redis;
use redis::Commands;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json;

use super::{Library, Peripheral};

/// Runs the callback functions for Model-specific setup in the database.
///
/// This method should be updated each time a new model is added to the Models module.
pub fn init(db: &redis::Connection) -> Result<()> {
    Library::init_count(&db)?;
    Peripheral::init_count(&db)?;

    Ok(())
}

pub trait Count {
    fn count(db: &redis::Connection) -> Result<usize>
    where
        Self: Query,
    {
        let count = redis::cmd("GET")
            .arg(format!("counters:{}", Self::key()))
            .query(db)
            .map_err(|e| DatabaseError { side: Box::new(e) })?;

        Ok(count)
    }

    fn count_and_incr(db: &redis::Connection) -> Result<usize>
    where
        Self: Query,
    {
        let count = Self::count(&db)?;
        Self::incr(&db)?;

        Ok(count)
    }

    fn incr(db: &redis::Connection) -> Result<()>
    where
        Self: Query,
    {
        redis::cmd("INCR")
            .arg(format!("counters:{}", Self::key()))
            .query(db)
            .map_err(|e| DatabaseError { side: Box::new(e) })?;

        Ok(())
    }

    fn init_count(db: &redis::Connection) -> Result<()>
    where
        Self: Query,
    {
        redis::cmd("SET")
            .arg(format!("counters:{}", Self::key()))
            .arg(0)
            .query(db)
            .map_err(|e| DatabaseError { side: Box::new(e) })?;

        Ok(())
    }
}

pub trait Query {
    fn all(db: &redis::Connection) -> Result<Vec<Self>>
    where
        Self: DeserializeOwned + Sized,
    {
        let keys: Vec<String> = db
            .scan_match(format!("{}:*", Self::key()))
            .map_err(|e| DatabaseError { side: Box::new(e) })?
            .collect();

        let json: Vec<String> = redis::cmd("MGET")
            .arg(keys)
            .query(db)
            .map_err(|e| DatabaseError { side: Box::new(e) })?;

        let mut result: Vec<Self> = Vec::new();
        for object in &json {
            result.push(
                serde_json::from_str(object).map_err(|e| DatabaseError { side: Box::new(e) })?,
            );
        }

        Ok(result)
    }

    fn id(&self) -> usize;

    fn get(db: &redis::Connection, id: usize) -> Result<Option<Self>>
    where
        Self: DeserializeOwned + Sized,
    {
        let result: Option<String> = redis::cmd("GET")
            .arg(format!("{}:{}", Self::key(), id))
            .query(db)
            .map_err(|e| DatabaseError { side: Box::new(e) })?;

        result
            .map(|result| {
                serde_json::from_str(&result).map_err(|e| DatabaseError { side: Box::new(e) })
            })
            .transpose()
    }

    fn key() -> &'static str;

    fn set(&self, db: &redis::Connection) -> Result<()>
    where
        Self: Serialize,
    {
        let json = serde_json::to_string(&self).map_err(|e| DatabaseError { side: Box::new(e) })?;

        redis::cmd("SET")
            .arg(format!("{}:{}", Self::key(), self.id()))
            .arg(json)
            .query(db)
            .map_err(|e| DatabaseError { side: Box::new(e) })?;

        Ok(())
    }

    fn set_id(&mut self, id: usize);
}

pub trait Queue {
    fn rpop(&self, db: &redis::Connection) -> Result<Option<String>>
    where
        Self: Query,
    {
        let msg: Option<String> = redis::cmd("RPOP")
            .arg(format!("queues:{}:{}", Self::key(), self.id()))
            .query(db)
            .map_err(|e| DatabaseError { side: Box::new(e) })?;

        Ok(msg)
    }
}

type Result<T> = std::result::Result<T, DatabaseError>;

#[derive(Debug)]
pub struct DatabaseError {
    side: Box<dyn Error>,
}

impl Error for DatabaseError {
    fn description(&self) -> &str {
        "Error when accessing the database"
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&*self.side)
    }
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DatabaseError {{ Cause: {} }}", &*self.side)
    }
}
