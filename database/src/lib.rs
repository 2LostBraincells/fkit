use sqlx::{migrate, AnyPool};

#[allow(unused)]
pub struct Database {
    pool: AnyPool,
}

impl Database {
    pub async fn new(url: &str) -> Database {
        let pool = sqlx::Pool::connect(url).await.unwrap();

        migrate!("./migrations").run(&pool).await.unwrap();

        Database { pool }
    }
}


