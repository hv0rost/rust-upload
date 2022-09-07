use sqlx::{Error, PgPool};

#[derive(Default)]
pub struct HashEntity {
    pub id : i32,
    pub hash : String,
    pub block_name : i32,
}

impl HashEntity {
    pub async fn get_hashes(pool: &PgPool) -> Result<Vec<HashEntity>, Error> {
        let result = sqlx::query_as!(HashEntity, "SELECT * FROM hash")
            .fetch_all(pool)
            .await?;

        Ok(result)
    }

    pub async fn get_last_block_name(pool: &PgPool) -> Result<i32, Error> {
        let result = sqlx::query!("SELECT MAX(block_name) FROM hash")
            .fetch_one(pool)
            .await?;

        Ok(result.max.unwrap())
    }

    pub async fn check_copy(pool: &PgPool, hash: String) -> Result<HashEntity, Error> {
        let result = sqlx::query_as!(HashEntity, "SELECT * FROM hash WHERE hash = $1", hash)
            .fetch_one(pool)
            .await?;

        Ok(result)
    }

    pub async fn create_new_hash(pool: &PgPool, hash : HashEntity) {
        sqlx::query!("INSERT INTO hash (hash, block_name) VALUES ($1, $2)",
            hash.hash,
            hash.block_name,
        ).fetch_one(pool).await.unwrap();
    }

    pub async fn delete_hash_data(pool: &PgPool, id : i32) {
        sqlx::query!("DELETE FROM hash WHERE id = $1", id).execute(pool).await.unwrap();
    }

    pub async fn update_hash_data(pool: &PgPool, hash : HashEntity) {
        sqlx::query!("UPDATE hash SET (hash, block_name) = ($1, $2) WHERE id=$3",
            hash.hash,
            hash.block_name,
            hash.id,
        ).execute(pool).await.unwrap();
    }
}