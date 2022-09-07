use sqlx::{Error, PgPool};
use serde_json::Value;

pub struct FileEntity {
    pub id : i32,
    pub filename : String,
    pub extension : String,
    pub mime : String,
    pub params : Option<Value>,
}


impl FileEntity {
    pub async fn get_files(pool: &PgPool) -> Result<Vec<FileEntity>, Error> {
        let result = sqlx::query_as!(FileEntity, "SELECT * FROM file")
            .fetch_all(pool)
            .await?;

        Ok(result)
    }

    pub async fn create_file_data(pool: &PgPool, file_data : FileEntity) {
        sqlx::query!("INSERT INTO file (filename, extension, mime, params) VALUES ($1, $2, $3, $4)",
            file_data.filename,
            file_data.extension,
            file_data.mime,
            file_data.params,
        ).fetch_one(pool).await.unwrap();
    }

    pub async fn delete_file_data(pool: &PgPool, id : i32) {
        sqlx::query!("DELETE FROM file WHERE id = $1", id).execute(pool).await.unwrap();
    }

    pub async fn update_file_data(pool: &PgPool, file_data : FileEntity) {
        sqlx::query!("UPDATE file SET (filename, extension, mime, params) = ($1, $2, $3, $4) WHERE id=$5",
            file_data.filename,
            file_data.extension,
            file_data.mime,
            file_data.params,
            file_data.id,
        ).execute(pool).await.unwrap();
    }
}