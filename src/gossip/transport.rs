use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use crate::gossip::message::Message;

///connect the addr to and send one message 
pub async fn send_message(addr: &str, msg: &Message) -> std::io::Result<()> {
    let mut stream = TcpStream::connect(addr).await?;
    write_message(&mut stream, msg).await
}

/// Write message onto an existing stream: [len][json]
pub async fn write_message(stream: &mut TcpStream, msg: &Message) -> std::io::Result<()> {
    let bytes = serde_json::to_vec(msg)?;
    stream.write_u32(bytes.len() as u32).await?;
    stream.write_all(&bytes).await?;
    stream.flush().await?;
    Ok(())
}

/// Read Exactly one length-prefixed message from the stream.
pub async fn read_message(stream: &mut TcpStream) -> std::io::Result<Message> {
    let len = stream.read_u32().await? as usize;
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).await?;
    let msg = serde_json::from_slice(&buf)?;
    Ok(msg)
}