use sqlx::PgPool;

pub async  fn create_connection_pool() -> PgPool {
    PgPool::connect("postgres://postgres:2254@localhost:5432/upload").await.unwrap()
}