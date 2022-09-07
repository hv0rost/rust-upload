use std::borrow::Borrow;
use std::convert::Infallible;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::ops::Add;
use bytes::{Buf, Bytes};
use data_encoding::HEXLOWER;
use futures::{Stream, TryStreamExt};
use mime::Mime;
use mpart_async::server::MultipartStream;
use sha2::{Sha256, Digest};
use uuid::Uuid;
use warp::{Error, Reply};

use crate::data_base::requests::{file, hash};
use crate::data_base::connection::create_connection_pool;

const BUFF_SIZE: usize = 1024 * 64;

pub(crate) async fn multi_part(
    mime: Mime,
    body: impl Stream<Item = Result<impl Buf, Error>> + Unpin,
) -> Result<impl Reply, Infallible> {
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

        let mut sized_bytes : Bytes;

        let mut last_block = hash::HashEntity::get_last_block_name(
            &create_connection_pool().await
        ).await.unwrap();

        while let Ok(Some(mut bytes)) = field.try_next().await {
            while bytes.len() > BUFF_SIZE{
                sized_bytes = bytes.copy_to_bytes(BUFF_SIZE);
                create_hash(&sized_bytes, &mut last_block).await;
                if bytes.len() < BUFF_SIZE {
                    sized_bytes = bytes.copy_to_bytes(bytes.remaining());
                    create_hash(&sized_bytes, &mut last_block).await;
                }
            }
            create_hash(&bytes, &mut last_block).await;
        }
        println!("{}", last_block)
    }

    Ok(format!("Success"))
}

async fn create_hash(bytes: &Bytes, mut last_block: &mut i32){
    let digest = {
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        hasher.finalize()
    };

    let hash = HEXLOWER.encode(digest.as_ref());

    let copy = hash::HashEntity::check_copy(&create_connection_pool().await, hash).await.unwrap_or_default();

    if copy.id == 0 {
        //hash::HashEntity::create_new_hash(&create_connection_pool().await, hash).await.unwrap_or_default();
    }

    *last_block += 1;
}

async fn create_block(bytes: &Bytes) {
    let file_name = format!("./uploads/{}", Uuid::new_v4().to_string());

    File::create(&file_name).unwrap();

    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(&file_name)
        .unwrap();

    file.write(&bytes).unwrap();
}