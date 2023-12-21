use cli::Command;
use handshake::handshake;
use std::env::{self};
use tokio::net::TcpStream;
use torrent_file::decode_torrent_file;
use tracker::tracker_get;

use crate::torrent_client::TorrentClient;

mod bencode;
mod cli;
mod handshake;
mod torrent_client;
mod torrent_file;
mod tracker;

// Usage: your_bittorrent.sh decode "<encoded_value>"
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let command_str = &args[1];

    let command = Command::from_str(command_str);
    if let None = command {
        println!("Unknown command: {}", args[1]);
        return Ok(());
    }

    let command = command.unwrap();
    match command {
        Command::Decode => {
            let _ = execute_command_decode(&args[2]);
        }
        Command::Info => {
            let _ = execute_command_info(&args[2]);
        }
        Command::Peers => {
            let _ = execute_command_peers(&args[2]).await;
        }
        Command::Handshake => {
            let _ = execute_command_handshake(&args[2]).await;
        }
        Command::DownloadPiece => {
            // "/tmp/codecrafters-bittorrent-target/release/bittorrent-starter-rust",
            // "download_piece",
            // "-o",
            // "/tmp/test-piece-0",
            // "sample.torrent",
            // "0"
            let input_file_path = &args[4];
            let output_file_path = &args[3];
            let piece_index: &u8 = &args[5].parse().unwrap();
            let _ =
                execute_command_download_piece(&input_file_path, &output_file_path, piece_index)
                    .await;
        }
    }

    Ok(())
}

// ---
// Commands bodies

fn execute_command_decode(encoded_value: &str) -> anyhow::Result<()> {
    let decoded_value = bencode::decode_bencoded_value(encoded_value)?;
    println!("{}", decoded_value);
    Ok(())
}

fn execute_command_info(file_path: &str) -> anyhow::Result<()> {
    let torrent = decode_torrent_file(file_path)?;
    println!("Tracker URL: {}", torrent.announce);
    println!("Length: {}", torrent.info.length);
    println!("Info Hash: {}", torrent.info.hash_hex()?);

    println!("Piece Length: {}", torrent.info.piece_length);
    println!("Piece Hashes:");
    torrent
        .info
        .pieces_hashes()?
        .iter()
        .for_each(|hash| println!("{}", hash));
    Ok(())
}

async fn execute_command_peers(file_path: &str) -> anyhow::Result<()> {
    let torrent = decode_torrent_file(file_path)?;
    let tracker_response = tracker_get(&torrent).await?;
    tracker_response
        .peers()
        .iter()
        .for_each(|peer| println!("{peer}"));
    Ok(())
}

async fn execute_command_handshake(file_path: &str) -> anyhow::Result<()> {
    let torrent = decode_torrent_file(file_path)?;
    let tracker_response = tracker_get(&torrent).await?;
    let peers = tracker_response.peers();
    let target_peer_address = peers.first().unwrap();
    let info_hash = torrent.info.hash_bytes()?;

    let mut stream = TcpStream::connect(target_peer_address).await?;
    let peer_id = handshake(&mut stream, &info_hash).await?;

    println!("Peer ID: {}", peer_id);
    Ok(())
}

async fn execute_command_download_piece(
    input_file_path: &str,
    output_file_path: &str,
    piece_index: &u8,
) -> anyhow::Result<()> {
    let mut client = TorrentClient::from_torrent_file(input_file_path).await?;
    client.fetch_peers().await?;
    client.connect().await?;
    client.handshake().await?;
    client.download_piece(piece_index, output_file_path).await?;
    Ok(())
}
