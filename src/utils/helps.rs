use std::convert::Infallible;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Write, Read, BufReader};
use bytes::Buf;
use futures::{Stream, TryStreamExt};
use mime::Mime;
use mpart_async::server::MultipartStream;
use sha2::{Sha256, Digest};
use sqlx::Error;
use tokio::io::AsyncReadExt;
use uuid::Uuid;
use warp::Reply;
use data_encoding::HEXLOWER;

pub(crate) async fn multi_part(
    mime: Mime,
    body: impl Stream<Item = Result<impl Buf, warp::Error>> + Unpin,
) -> Result<impl Reply, Infallible> {
    let boundary = mime.get_param("boundary").map(|v| v.to_string()).unwrap();

    let mut stream = MultipartStream::new(
        boundary,
        body.map_ok(|mut buf| buf.copy_to_bytes(buf.remaining())),
    );


    while let Ok(Some(mut field)) = stream.try_next().await {
        println!("Field received:{}", field.name().unwrap());
        if let Ok(filename) = field.filename() {
            println!("Field filename:{}", filename);
        }

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

        let file_name = format!("./uploads/{}.{}", Uuid::new_v4().to_string(), file_ending);

        File::create(&file_name).unwrap();

        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&file_name)
            .unwrap();

        while let Ok(Some(mut bytes)) = field.try_next().await {
            //println!("bytes readed: {}", bytes.len());
            file.write(&mut bytes).unwrap();
        }
        println!("{}", create_hash(file_name).await.unwrap());
    }

    Ok(format!("Success"))
}

async fn create_hash(path: String) -> Result<String, Error> {
    let input = File::open(path)?;
    let mut reader = BufReader::new(input);

    let digest = {
        let mut hasher = Sha256::new();
        let mut buffer = [0; 16384];
        loop {
            let count = reader.read(&mut buffer)?;
            if count == 0 { break }
            hasher.update(&buffer[..count]);
        }
        hasher.finalize()
    };
    Ok(HEXLOWER.encode(digest.as_ref()))
}