use r2d2_postgres::{
    r2d2::{Pool, PooledConnection},
    PostgresConnectionManager, TlsMode,
};
use std::fmt::Debug;

type Connection = PooledConnection<PostgresConnectionManager>;

pub(crate) struct DataInfra {
    pool: Pool<PostgresConnectionManager>,
}

fn get_uri() -> String {
    std::env::var("DB_URI").unwrap()
}

fn handle_err<E: Debug>(err: E) {
    error!("{:?}", err);
}

const MAX_CONNECTION_COUNT: u32 = 3;

impl DataInfra {
    pub(crate) fn new() -> Result<Self, ()> {
        let manager =
            PostgresConnectionManager::new(get_uri(), TlsMode::None).map_err(handle_err)?;
        let pool = Pool::builder()
            .min_idle(Some(0))
            .max_size(MAX_CONNECTION_COUNT)
            .build(manager)
            .map_err(handle_err)?;

        let it = DataInfra { pool };
        it.initialize()?;

        Ok(it)
    }

    fn connect(&self) -> Result<Connection, ()> {
        self.pool.get().map_err(handle_err)
    }

    fn initialize(&self) -> Result<(), ()> {
        let connection = self.connect()?;
        match connection.batch_execute(
            r#"
                create table entries(
                    key varchar(1024) primary key,
                    value varchar(4096) not null,
                    created_at timestamp default current_timestamp not null
                );

                create index on entries(created_at);
            "#,
        ) {
            Err(ref err) if format!("{:?}", err).contains("already exists") => {
                // OK.
                Ok(())
            }
            Ok(_) => {
                // Success.
                Ok(())
            }
            Err(err) => panic!("{}", err),
        }
    }

    pub(crate) fn insert(&self, key: String, value: String) -> Result<(), ()> {
        let connection = self.connect()?;

        // FIXME: use prepared statement
        connection
            .execute(
                r#"
                    insert into entries(key, value)
                        values ($1, $2)
                "#,
                &[&key, &value],
            )
            .map_err(handle_err)?;

        Ok(())
    }

    pub(crate) fn delete(&self, key: &str) -> Result<(), ()> {
        let connection = self.connect()?;

        // FIXME: use prepared statement
        connection
            .execute(
                r#"
                    delete from entries
                    where key = $1
                "#,
                &[&key.to_string()],
            )
            .map_err(handle_err)?;

        Ok(())
    }

    pub(crate) fn find(&self, key: &str) -> Result<Option<String>, ()> {
        let connection = self.connect()?;

        let rows = connection
            .query(
                "select value from entries where key = $1",
                &[&key.to_string()],
            )
            .map_err(handle_err)?;

        Ok(rows.iter().next().map(|row| {
            let value: String = row.get(0);
            value
        }))
    }

    pub(crate) fn count(&self) -> Result<usize, ()> {
        let connection = self.connect()?;

        let rows = connection
            .query("select count(*) from entries", &[])
            .map_err(handle_err)?;
        let count = rows
            .iter()
            .next()
            .map(|row| -> i64 { row.get(0) })
            .unwrap_or(0);

        Ok(count as usize)
    }

    pub(crate) fn delete_old_entries(&self, retain_count: usize) -> Result<(), ()> {
        let connection = self.connect()?;
        connection
            .execute(
                r#"
                    delete from entries
                    where key not in (
                        select key
                        from entries
                        order by created_at desc
                        limit $1
                    )
                "#,
                &[&(retain_count as i64)],
            )
            .map_err(handle_err)?;

        Ok(())
    }
}
