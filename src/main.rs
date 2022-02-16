use async_std::net::{TcpStream, ToSocketAddrs};
use async_std::prelude::*;

#[async_std::main]
async fn main() -> async_std::io::Result<()> {
    let mut client = Client::new("127.0.0.1:6379").await?;
    client.set("name".to_string(), "Dogukan".to_string()).await.unwrap();

    let response = client.get("name".to_string()).await.unwrap();


    println!("{}", response);

    Ok(())
}

struct Client {
    stream: TcpStream
}

impl Client {
    async fn new<A: ToSocketAddrs>(addr: A) -> Result<Client, async_std::io::Error> {
        let stream = TcpStream::connect(addr).await?;
        Ok(Self { stream })
    }
    
    async fn get(&mut self, key: String) -> Result<String, String> {
        let command = RespType::Array(vec![
            RespType::BulkString(b"GET".to_vec()),
            RespType::BulkString(key.as_bytes().to_vec())
        ]);
        
        self.run(command).await
    }

    async fn set(&mut self, key: String, value: String) -> Result<String, String> {
        let command = RespType::Array(vec![
            RespType::BulkString(b"SET".to_vec()),
            RespType::BulkString(key.as_bytes().to_vec()),
            RespType::BulkString(value.as_bytes().to_vec())
        ]);
        
        self.run(command).await
    }

    async fn run(&mut self, data: RespType) -> Result<String, String> {
        let mut write_buffer = vec![];
        
        data.serialize(&mut write_buffer);
        self.stream.write_all(&write_buffer).await.unwrap();

        let mut read_buffer = vec![0; 1024];
        let bytes_read = self.stream.read(&mut read_buffer).await.unwrap();

        self.parse_response(&read_buffer[..bytes_read])
    }

    fn parse_response(&self, buffer: &[u8]) -> Result<String, String> {
        if buffer.is_empty() {
            return Err(String::from("Buffer is empty!"));
        }

        let response = std::str::from_utf8(&buffer).unwrap();

        return match buffer[0] {
            b'-' => Err(String::from(format!("An error occured: {}", &response[1..buffer.len() - 2]))),
            b'+' => Ok(response[1..buffer.len() - 2].to_owned()),
            b'$' => {
                if let Some(i) = response.find("\n") {
                    // return the value after $byte\r\n
                    return Ok(response[i + 1..buffer.len() - 2].to_owned());
                }

                return Err(String::from("An error occured while parsing the response"));
            },
            _ => Ok(response.to_owned())
        };
    }
}

enum RespType {
    SimpleString(String),
    Error(Vec<u8>),
    Integer(i64),
    BulkString(Vec<u8>),
    Array(Vec<RespType>)
}

impl RespType {
    fn serialize(self, buffer: &mut Vec<u8>) {
        match self {
            RespType::Array(values) => {
                buffer.push(b'*');
                buffer.append(&mut values.len().to_string().into_bytes());
                buffer.append(&mut b"\r\n".to_vec());

                for value in values {
                    value.serialize(buffer);
                }
            },
            RespType::BulkString(mut value) => {
                buffer.push(b'$');
                buffer.append(&mut value.len().to_string().into_bytes());
                buffer.append(&mut b"\r\n".to_vec());
                buffer.append(&mut value);
                buffer.append(&mut b"\r\n".to_vec());
            }
            _ => unimplemented!()
        }
    }
}