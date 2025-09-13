//! BitSwapTorrent wire protocol implementation
//!
//! BitTorrent benzeri ancak modern bir mesajlaşma protokolü

use crate::error::{BitSwapError, BitSwapResult};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use uuid::Uuid;

/// Protocol version
pub const PROTOCOL_VERSION: u8 = 1;

/// Protocol identifier
pub const PROTOCOL_ID: &[u8] = b"BITSWAP-1-SHA256";

/// Message frame yapısı: [length: u32 BE][msg_id: u8][payload...]
#[derive(Debug, Clone)]
pub struct MessageFrame {
    /// Message ID
    pub id: u8,
    /// Payload data
    pub payload: Bytes,
}

/// Wire protocol messages
#[derive(Debug, Clone)]
pub enum Message {
    /// Choke message (suspend uploading to peer)
    Choke,
    /// Unchoke message (resume uploading to peer)
    Unchoke,
    /// Interested message (want to download from peer)
    Interested,
    /// Not interested message
    NotInterested,
    /// Have message (announce piece availability)
    Have { piece_index: u32 },
    /// Bitfield message (announce which pieces we have)
    Bitfield { bitfield: BitField },
    /// Request message (request piece data)
    Request {
        piece_index: u32,
        begin: u32,
        length: u32,
    },
    /// Piece message (send piece data)
    Piece {
        piece_index: u32,
        begin: u32,
        data: Bytes,
    },
    /// Cancel message (cancel previous request)
    Cancel {
        piece_index: u32,
        begin: u32,
        length: u32,
    },
    /// Keepalive message
    Keepalive,
    /// Extended message (for future extensions)
    Extended { extension_id: u8, payload: Bytes },
    /// Handshake message (special case, not part of frame protocol)
    Handshake(HandshakeMessage),
}

/// Message IDs
impl Message {
    pub const CHOKE: u8 = 0;
    pub const UNCHOKE: u8 = 1;
    pub const INTERESTED: u8 = 2;
    pub const NOT_INTERESTED: u8 = 3;
    pub const HAVE: u8 = 4;
    pub const BITFIELD: u8 = 5;
    pub const REQUEST: u8 = 6;
    pub const PIECE: u8 = 7;
    pub const CANCEL: u8 = 8;
    pub const KEEPALIVE: u8 = 9;
    pub const EXTENDED: u8 = 20;
}

/// Bitfield for tracking piece availability
#[derive(Debug, Clone)]
pub struct BitField {
    data: Vec<u8>,
    num_pieces: u32,
}

impl BitField {
    /// Create new empty bitfield
    pub fn new(num_pieces: u32) -> Self {
        let byte_count = (num_pieces + 7) / 8;
        Self {
            data: vec![0u8; byte_count as usize],
            num_pieces,
        }
    }

    /// Create from raw bytes
    pub fn from_bytes(data: Vec<u8>, num_pieces: u32) -> Self {
        Self { data, num_pieces }
    }

    /// Set piece as available
    pub fn set_piece(&mut self, piece_index: u32) -> BitSwapResult<()> {
        if piece_index >= self.num_pieces {
            return Err(BitSwapError::InvalidPieceIndex {
                index: piece_index,
                total: self.num_pieces,
            });
        }

        let byte_index = (piece_index / 8) as usize;
        let bit_index = (piece_index % 8) as usize;
        
        if byte_index < self.data.len() {
            self.data[byte_index] |= 1 << (7 - bit_index);
        }
        
        Ok(())
    }

    /// Check if piece is available
    pub fn has_piece(&self, piece_index: u32) -> bool {
        if piece_index >= self.num_pieces {
            return false;
        }

        let byte_index = (piece_index / 8) as usize;
        let bit_index = (piece_index % 8) as usize;
        
        if byte_index < self.data.len() {
            (self.data[byte_index] & (1 << (7 - bit_index))) != 0
        } else {
            false
        }
    }

    /// Get raw bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Count number of pieces available
    pub fn count_pieces(&self) -> u32 {
        let mut count = 0;
        for i in 0..self.num_pieces {
            if self.has_piece(i) {
                count += 1;
            }
        }
        count
    }

    /// Check if we have all pieces
    pub fn is_complete(&self) -> bool {
        self.count_pieces() == self.num_pieces
    }
}

/// Handshake message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeMessage {
    /// Protocol identifier
    pub protocol: String,
    /// Protocol version
    pub version: u8,
    /// Info hash (32 bytes SHA-256)
    pub info_hash: [u8; 32],
    /// Peer ID (16 bytes UUID)
    pub peer_id: Uuid,
    /// Supported capabilities bitmask
    pub capabilities: u64,
}

impl HandshakeMessage {
    /// Create new handshake message
    pub fn new(info_hash: [u8; 32], peer_id: Uuid) -> Self {
        Self {
            protocol: PROTOCOL_ID.iter().map(|&b| b as char).collect(),
            version: PROTOCOL_VERSION,
            info_hash,
            peer_id,
            capabilities: 0, // No extensions yet
        }
    }

    /// Serialize handshake to bytes
    pub fn to_bytes(&self) -> BitSwapResult<Vec<u8>> {
        let mut buf = Vec::new();
        
        // Protocol string length + protocol string
        let protocol_bytes = self.protocol.as_bytes();
        buf.put_u8(protocol_bytes.len() as u8);
        buf.extend_from_slice(protocol_bytes);
        
        // Version
        buf.put_u8(self.version);
        
        // Info hash (32 bytes)
        buf.extend_from_slice(&self.info_hash);
        
        // Peer ID (16 bytes)
        buf.extend_from_slice(self.peer_id.as_bytes());
        
        // Capabilities (8 bytes)
        buf.put_u64(self.capabilities);
        
        Ok(buf)
    }

    /// Deserialize handshake from bytes
    pub fn from_bytes(data: &[u8]) -> BitSwapResult<Self> {
        if data.len() < 58 { // Minimum size
            return Err(BitSwapError::Protocol("Handshake too short".to_string()));
        }

        let mut cursor = Cursor::new(data);
        
        // Read protocol string
        let protocol_len = cursor.get_u8() as usize;
        if cursor.remaining() < protocol_len + 57 {
            return Err(BitSwapError::Protocol("Invalid handshake length".to_string()));
        }
        
        let mut protocol_bytes = vec![0u8; protocol_len];
        cursor.copy_to_slice(&mut protocol_bytes);
        let protocol = String::from_utf8(protocol_bytes)
            .map_err(|_| BitSwapError::Protocol("Invalid protocol string".to_string()))?;

        // Version
        let version = cursor.get_u8();
        
        // Info hash
        let mut info_hash = [0u8; 32];
        cursor.copy_to_slice(&mut info_hash);
        
        // Peer ID
        let mut peer_id_bytes = [0u8; 16];
        cursor.copy_to_slice(&mut peer_id_bytes);
        let peer_id = Uuid::from_bytes(peer_id_bytes);
        
        // Capabilities
        let capabilities = cursor.get_u64();

        Ok(Self {
            protocol,
            version,
            info_hash,
            peer_id,
            capabilities,
        })
    }
}

impl Message {
    /// Serialize message to frame
    pub fn to_frame(&self) -> BitSwapResult<MessageFrame> {
        match self {
            Message::Choke => Ok(MessageFrame {
                id: Self::CHOKE,
                payload: Bytes::new(),
            }),
            
            Message::Unchoke => Ok(MessageFrame {
                id: Self::UNCHOKE,
                payload: Bytes::new(),
            }),
            
            Message::Interested => Ok(MessageFrame {
                id: Self::INTERESTED,
                payload: Bytes::new(),
            }),
            
            Message::NotInterested => Ok(MessageFrame {
                id: Self::NOT_INTERESTED,
                payload: Bytes::new(),
            }),
            
            Message::Have { piece_index } => {
                let mut payload = BytesMut::with_capacity(4);
                payload.put_u32(*piece_index);
                Ok(MessageFrame {
                    id: Self::HAVE,
                    payload: payload.freeze(),
                })
            }
            
            Message::Bitfield { bitfield } => Ok(MessageFrame {
                id: Self::BITFIELD,
                payload: Bytes::copy_from_slice(bitfield.as_bytes()),
            }),
            
            Message::Request { piece_index, begin, length } => {
                let mut payload = BytesMut::with_capacity(12);
                payload.put_u32(*piece_index);
                payload.put_u32(*begin);
                payload.put_u32(*length);
                Ok(MessageFrame {
                    id: Self::REQUEST,
                    payload: payload.freeze(),
                })
            }
            
            Message::Piece { piece_index, begin, data } => {
                let mut payload = BytesMut::with_capacity(8 + data.len());
                payload.put_u32(*piece_index);
                payload.put_u32(*begin);
                payload.extend_from_slice(data);
                Ok(MessageFrame {
                    id: Self::PIECE,
                    payload: payload.freeze(),
                })
            }
            
            Message::Cancel { piece_index, begin, length } => {
                let mut payload = BytesMut::with_capacity(12);
                payload.put_u32(*piece_index);
                payload.put_u32(*begin);
                payload.put_u32(*length);
                Ok(MessageFrame {
                    id: Self::CANCEL,
                    payload: payload.freeze(),
                })
            }
            
            Message::Keepalive => Ok(MessageFrame {
                id: Self::KEEPALIVE,
                payload: Bytes::new(),
            }),
            
            Message::Extended { extension_id, payload } => {
                let mut frame_payload = BytesMut::with_capacity(1 + payload.len());
                frame_payload.put_u8(*extension_id);
                frame_payload.extend_from_slice(payload);
                Ok(MessageFrame {
                    id: Self::EXTENDED,
                    payload: frame_payload.freeze(),
                })
            }
            
            Message::Handshake(_) => {
                Err(BitSwapError::Protocol("Handshake is not a regular frame message".to_string()))
            }
        }
    }

    /// Parse message from frame
    pub fn from_frame(frame: &MessageFrame) -> BitSwapResult<Self> {
        let mut payload = Cursor::new(&frame.payload);
        
        match frame.id {
            Self::CHOKE => Ok(Message::Choke),
            Self::UNCHOKE => Ok(Message::Unchoke),
            Self::INTERESTED => Ok(Message::Interested),
            Self::NOT_INTERESTED => Ok(Message::NotInterested),
            
            Self::HAVE => {
                if payload.remaining() < 4 {
                    return Err(BitSwapError::Protocol("Invalid HAVE payload".to_string()));
                }
                Ok(Message::Have {
                    piece_index: payload.get_u32(),
                })
            }
            
            Self::BITFIELD => {
                let bitfield_data = frame.payload.to_vec();
                // We don't know the exact piece count here, so we estimate
                let estimated_pieces = bitfield_data.len() as u32 * 8;
                Ok(Message::Bitfield {
                    bitfield: BitField::from_bytes(bitfield_data, estimated_pieces),
                })
            }
            
            Self::REQUEST => {
                if payload.remaining() < 12 {
                    return Err(BitSwapError::Protocol("Invalid REQUEST payload".to_string()));
                }
                Ok(Message::Request {
                    piece_index: payload.get_u32(),
                    begin: payload.get_u32(),
                    length: payload.get_u32(),
                })
            }
            
            Self::PIECE => {
                if payload.remaining() < 8 {
                    return Err(BitSwapError::Protocol("Invalid PIECE payload".to_string()));
                }
                let piece_index = payload.get_u32();
                let begin = payload.get_u32();
                let data = Bytes::copy_from_slice(&frame.payload[8..]);
                Ok(Message::Piece {
                    piece_index,
                    begin,
                    data,
                })
            }
            
            Self::CANCEL => {
                if payload.remaining() < 12 {
                    return Err(BitSwapError::Protocol("Invalid CANCEL payload".to_string()));
                }
                Ok(Message::Cancel {
                    piece_index: payload.get_u32(),
                    begin: payload.get_u32(),
                    length: payload.get_u32(),
                })
            }
            
            Self::KEEPALIVE => Ok(Message::Keepalive),
            
            Self::EXTENDED => {
                if payload.remaining() < 1 {
                    return Err(BitSwapError::Protocol("Invalid EXTENDED payload".to_string()));
                }
                let extension_id = payload.get_u8();
                let extension_payload = Bytes::copy_from_slice(&frame.payload[1..]);
                Ok(Message::Extended {
                    extension_id,
                    payload: extension_payload,
                })
            }
            
            _ => Err(BitSwapError::Protocol(format!("Unknown message ID: {}", frame.id))),
        }
    }
}

impl MessageFrame {
    /// Write frame to async writer
    pub async fn write_to<W>(&self, writer: &mut W) -> BitSwapResult<()>
    where
        W: AsyncWriteExt + Unpin,
    {
        let payload_len = self.payload.len() as u32;
        let total_len = 1 + payload_len; // msg_id + payload
        
        // Write length (big endian)
        writer.write_u32(total_len).await?;
        
        // Write message ID
        writer.write_u8(self.id).await?;
        
        // Write payload
        if !self.payload.is_empty() {
            writer.write_all(&self.payload).await?;
        }
        
        writer.flush().await?;
        Ok(())
    }

    /// Read frame from async reader
    pub async fn read_from<R>(reader: &mut R) -> BitSwapResult<Option<Self>>
    where
        R: AsyncReadExt + Unpin,
    {
        // Read length
        let length = match reader.read_u32().await {
            Ok(len) => len,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(e.into()),
        };

        // Keepalive (length = 0)
        if length == 0 {
            return Ok(Some(Self {
                id: Message::KEEPALIVE,
                payload: Bytes::new(),
            }));
        }

        // Read message ID
        let msg_id = reader.read_u8().await?;
        
        // Read payload
        let payload_len = length - 1; // subtract msg_id byte
        let mut payload_buf = vec![0u8; payload_len as usize];
        
        if payload_len > 0 {
            reader.read_exact(&mut payload_buf).await?;
        }
        
        Ok(Some(Self {
            id: msg_id,
            payload: Bytes::from(payload_buf),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_bitfield_operations() {
        let mut bitfield = BitField::new(20);
        
        assert!(!bitfield.has_piece(5));
        bitfield.set_piece(5).unwrap();
        assert!(bitfield.has_piece(5));
        
        bitfield.set_piece(10).unwrap();
        bitfield.set_piece(15).unwrap();
        
        assert_eq!(bitfield.count_pieces(), 3);
        assert!(!bitfield.is_complete());
    }

    #[test]
    fn test_handshake_serialization() {
        let info_hash = [0u8; 32];
        let peer_id = Uuid::new_v4();
        let handshake = HandshakeMessage::new(info_hash, peer_id);
        
        let bytes = handshake.to_bytes().unwrap();
        let parsed = HandshakeMessage::from_bytes(&bytes).unwrap();
        
        assert_eq!(parsed.peer_id, peer_id);
        assert_eq!(parsed.info_hash, info_hash);
        assert_eq!(parsed.version, PROTOCOL_VERSION);
    }

    #[tokio::test]
    async fn test_message_frame_io() {
        let message = Message::Have { piece_index: 42 };
        let frame = message.to_frame().unwrap();
        
        // Write to buffer
        let mut buf = Vec::new();
        {
            let mut cursor = Cursor::new(&mut buf);
            frame.write_to(&mut cursor).await.unwrap();
        }
        
        // Read back
        let mut cursor = Cursor::new(&buf);
        let read_frame = MessageFrame::read_from(&mut cursor).await.unwrap().unwrap();
        
        assert_eq!(read_frame.id, frame.id);
        assert_eq!(read_frame.payload, frame.payload);
        
        // Parse message
        let parsed_message = Message::from_frame(&read_frame).unwrap();
        match parsed_message {
            Message::Have { piece_index } => assert_eq!(piece_index, 42),
            _ => panic!("Wrong message type"),
        }
    }
}
