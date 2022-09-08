use std::convert::Infallible;
use std::fs::{File, OpenOptions};
use std::io::Write;
use bytes::{Buf, Bytes};
use data_encoding::HEXLOWER;
use futures::{Stream, TryStreamExt};
use mime::Mime;
use mpart_async::server::MultipartStream;
use sha2::{Sha256, Digest};
use sqlx::PgPool;
use uuid::Uuid;
use warp::{Error, Reply};

use crate::data_base::requests::{file, hash};
use crate::data_base::connection::create_connection_pool;

const BUFF_SIZE: usize = 1024 * 64;

pub(crate) async fn multi_part(
    mime: Mime,
    body: impl Stream<Item = Result<impl Buf, Error>> + Unpin,
) -> Result<impl Reply, Infallible> {
    let pool = create_connection_pool().await;
    let boundary = mime.get_param("boundary").map(|v| v.to_string()).unwrap();

    let mut stream = MultipartStream::new(
        boundary,
        body.map_ok(|mut buf| buf.copy_to_bytes(buf.remaining())),
    );

    while let Ok(Some(mut field)) = stream.try_next().await {
        field.name().unwrap();

        let file_ending;
        match field.content_type().unwrap() {
            "text/plain" => {
                file_ending = "txt";
            }
            "image/jpeg" => {
                file_ending = "jpg";
            }
            "image/png" => {
                file_ending = "png";
            }
            "video/mp4" => {
                file_ending = "mp4";
            }
            v => {
                return  Ok(format!("Unsupported media type: {}", v));
            }
        }

        let file_data = file::FileEntity{
            id: 0,
            filename: Uuid::new_v4().to_string(),
            extension: file_ending.to_string(),
            mime: field.content_type().unwrap().to_string(),
            params: None
        };

        let mut sized_bytes = Bytes::new();
        let mut difference : usize;

        let mut last_block = hash::HashEntity::get_last_block_name(&pool).await.unwrap();
        while let Ok(Some(mut bytes)) = field.try_next().await {
            println!("input bytes {}", bytes.len());
            sized_bytes = Bytes::from([sized_bytes.clone(), bytes.copy_to_bytes(bytes.remaining())].concat());

            println!("before oversize {}", sized_bytes.len());
            if sized_bytes.len() >= BUFF_SIZE{
                difference = sized_bytes.len() - BUFF_SIZE;
                bytes = sized_bytes.copy_to_bytes(difference);
                //create_hash(&pool, &sized_bytes, &mut last_block).await;
                println!("after oversize {}", sized_bytes.len());
            }
            println!("{}", bytes.len());
            while bytes.len() > BUFF_SIZE{
                println!(">>>{}", bytes.len());
                sized_bytes = bytes.copy_to_bytes(BUFF_SIZE);
                //create_hash(&pool, &sized_bytes, &mut last_block).await;
                if bytes.len() < BUFF_SIZE {
                    sized_bytes = bytes.copy_to_bytes(bytes.remaining());
                    //create_hash(&pool, &sized_bytes, &mut last_block).await;
                }
            }
            //create_hash(&pool, &bytes, &mut last_block).await;
        }
    }

    Ok(format!("Success"))
}

async fn create_hash(pool : &PgPool, bytes: &Bytes, mut last_block: &mut i32){
    let digest = {
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        hasher.finalize()
    };

    *last_block += 1;

    let hash = HEXLOWER.encode(digest.as_ref());

    let copy = hash::HashEntity::check_copy(pool, &hash).await.unwrap_or_default();

    if copy.id == 0 {
        hash::HashEntity::create_new_hash(pool, hash, last_block.clone()).await;
        create_block(bytes, last_block.clone()).await;
    }
}

async fn create_block(bytes: &Bytes, last_block: i32) {
    let file_name = format!("./uploads/{}", last_block);

    File::create(&file_name).unwrap();

    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(&file_name)
        .unwrap();

    file.write(&bytes).unwrap();
}