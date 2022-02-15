use async_std::net::TcpStream;
use async_std::prelude::*;

#[async_std::main]
async fn main() -> async_std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:6379").await?;

    let command = b"*1\r\n$4\r\nPING\r\n";
    stream.write_all(command).await?;
    let mut buffer = vec![0; 1024];
    let bytes_read = stream.read(&mut buffer).await?;
    println!("{:?}", std::str::from_utf8(&buffer[..bytes_read]));

    Ok(())
}
