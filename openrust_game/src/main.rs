extern crate core;

use std::io::{self, Cursor, Error, ErrorKind, Read};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use bytes::{Buf, BufMut, BytesMut};
use futures::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{Decoder, Encoder, Framed};
use openrust_fs::cache::Cache;
use openrust_fs::checksum_table::ChecksumTable;
use openrust_fs::container::{self, Container};
use openrust_fs::filestore::FileStore;

const HANDSHAKE_UPDATE: u8 = 15;
const VERSION: u32 = 530;

const STATUS_OK: u8 = 0;
const STATUS_OUT_OF_DATE: u8 = 6;

#[derive(Debug)]
pub struct GameServer {
    cache: Arc<Mutex<Cache>>,
    checksum_table: ChecksumTable,
}

impl GameServer {
    pub fn new() -> io::Result<Self> {
        let cache = Arc::new(Mutex::new(Cache::new(FileStore::open("openrust_data/fs/")?)));
        let checksum_table = cache.lock().unwrap().create_checksum_table()?;

        Ok(Self { cache, checksum_table })
    }
}

#[derive(Debug)]
pub enum GameMessage {
    UpdateStatus { status_id: u8 },
    FileResponse { type_id: u8, file_id: u16, priority: bool, container: Cursor<Vec<u8>> },
}

enum GameState {
    Handshake,
    Update,
}

pub struct GameDecoder {
    state: GameState,
    server: Arc<GameServer>,
}

impl GameDecoder {
    pub fn new(server: Arc<GameServer>) -> Self {
        Self { state: GameState::Handshake, server }
    }
}

impl Decoder for GameDecoder {
    type Item = GameMessage;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match self.state {
            GameState::Handshake => {
                let service_id = src.get_u8();
                return match service_id {
                    HANDSHAKE_UPDATE => {
                        let version = src.get_u32();
                        let status_id = if version != VERSION { STATUS_OUT_OF_DATE } else { STATUS_OK };
                        if status_id == STATUS_OK {
                            self.state = GameState::Update;
                        }

                        Ok(Some(GameMessage::UpdateStatus { status_id }))
                    }
                    _ => Err(Error::new(ErrorKind::InvalidData, "Invalid Handshake Service Id!"))
                };
            }
            GameState::Update => {
                while src.has_remaining()  {
                    let opcode = src.get_u8();
                    match opcode {
                        0 | 1 => {
                            if src.remaining() < 3 {
                                /*
                                  todo: strange edge-case when I get to "loading interfaces" there's
                                   not enough bytes in the buffer... this seems to be a "hacky" way
                                   of bypassing it but I need to figure out the cause behind this.
                                 */
                                return Ok(None);
                            }

                            let type_id = src.get_u8();
                            let file_id = src.get_u16();
                            let priority = opcode == 1;
                            let container = if type_id == 255 && file_id == 255 {
                                let table = &self.server.checksum_table;
                                let container = Container::new(container::COMPRESSION_NONE, table.encode()?);
                                Cursor::new(container.encode()?.into_inner())
                            } else {
                                let mut cache = self.server.cache.lock().expect("Failed to acquire lock");
                                let mut data = cache.store_mut().read(type_id as usize, file_id as usize)?.into_inner();

                                if type_id != 255 {
                                    let len = data.len();
                                    data.truncate(len - 2);
                                }

                                Cursor::new(data)
                            };

                            return Ok(Some(GameMessage::FileResponse { type_id, file_id, priority, container }));
                        }
                        _ => {
                            src.advance(3);
                        }
                    };
                }
            }
        }

        Ok(None)
    }
}

impl Encoder<GameMessage> for GameDecoder {
    type Error = Error;

    fn encode(&mut self, item: GameMessage, dst: &mut BytesMut) -> Result<(), Self::Error> {
        match item {
            GameMessage::UpdateStatus { status_id } => {
                dst.put_u8(status_id);
            }
            GameMessage::FileResponse { type_id, file_id, priority, mut container } => {
                dst.put_u8(type_id);
                dst.put_u16(file_id);

                let mut compression = container.get_u8();
                if !priority {
                    compression |= 0x80;
                }

                dst.put_u8(compression);

                let mut bytes = container.remaining();
                if bytes > 508 {
                    bytes = 508;
                }

                let mut buffer = vec![0; bytes];
                container.read_exact(&mut buffer)?;

                dst.put(buffer.as_slice());

                loop {
                    let mut bytes = container.remaining();
                    if bytes == 0 {
                        break;
                    } else if bytes > 511 {
                        bytes = 511;
                    }

                    let mut buffer = vec![0; bytes];
                    container.read_exact(&mut buffer)?;

                    dst.put_u8(0xFF);
                    dst.put(buffer.as_slice());
                }
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let server = Arc::new(GameServer::new()?);
    let addr = "127.0.0.1:43594".to_string().parse::<SocketAddr>().unwrap();
    let listener = TcpListener::bind(&addr).await?;
    println!("Listening for connections on: {}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let framed = Framed::new(stream, GameDecoder::new(Arc::clone(&server)));

        tokio::spawn(handle_client(framed));
    }
}

async fn handle_client(mut framed: Framed<TcpStream, GameDecoder>) -> io::Result<()> {
    while let Some(message) = framed.next().await {
        match message {
            Ok(message) => framed.send(message).await?,
            Err(e) => return Err(e)
        }
    }

    Ok(())
}