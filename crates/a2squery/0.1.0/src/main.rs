use std::net::SocketAddr;

use anyhow::Context;
use clap::Parser;
use serde::Serialize;
use tokio::net::UdpSocket;

#[derive(Parser,Debug)]
#[command(name = "a2squery", author, version, about)]
struct Args {
	/// Server hostname or IP
	#[arg(long)]
	host: String,

	/// The server's query port
	#[arg(long)]
	port: u16
}

#[derive(Serialize,Debug)]
enum ServerType {
	Dedicated,
	Listen,
	SourceTV
}

#[derive(Serialize,Debug)]
enum OsType {
	Linux,
	Windows,
	Mac
}

#[derive(Serialize,Debug)]
enum Visibility {
	Public,
	Private
}

#[derive(Serialize,Debug)]
enum A2SExtraData {
	ServerPort(u16),
	SteamID(u64),
	STVPortAndName(u16,String),
	Keywords(String),
	GameID(u64)
}

impl A2SExtraData {
	pub fn from_bytes(edf: u8, buf: &[u8]) -> anyhow::Result<Vec<Self>> {
		let mut data = vec![];
		let mut cursor = 0;
		if edf & 0x80 == 1 {
			let port = u16::from_le_bytes([buf[cursor], buf[cursor + 1]]);
			data.push(Self::ServerPort(port));
			cursor += 2;
		}

		if edf & 0x10 == 1 {
			let steam_id = u64::from_le_bytes([
				buf[cursor], buf[cursor + 1], buf[cursor + 2], buf[cursor + 3],
				buf[cursor + 4], buf[cursor + 5], buf[cursor + 6], buf[cursor + 7]
			]);
			cursor += 8;
			data.push(Self::SteamID(steam_id));
		}

		if edf & 0x40 == 1 {
			let stv_port = u16::from_le_bytes([buf[cursor], buf[cursor + 1]]);
			cursor += 2;
			let mut stv_name = String::new();
			loop {
				let byte = buf[cursor];
				if byte == b'\x00' {
					cursor += 1;
					break
				}
				stv_name.push(byte as char);
				cursor += 1;
			}
			data.push(Self::STVPortAndName(stv_port, stv_name));
		}

		if edf & 0x20 == 1 {
			let mut keywords = String::new();
			loop {
				let byte = buf[cursor];
				if byte == b'\x00' {
					cursor += 1;
					break
				}
				keywords.push(byte as char);
				cursor += 1;
			}
			data.push(Self::Keywords(keywords));
		}

		if edf & 0x01 == 1 {
			let game_id = u64::from_le_bytes([
				buf[cursor], buf[cursor + 1], buf[cursor + 2], buf[cursor + 3],
				buf[cursor + 4], buf[cursor + 5], buf[cursor + 6], buf[cursor + 7]
			]);
			data.push(Self::GameID(game_id));
		}
		Ok(data)
	}
}

#[derive(Serialize,Debug)]
struct A2SInfo {
	protocol_ver: u8,
	server_name: String,
	map_name: String,
	game_dir: String,
	game_name: String,
	app_id: u16,
	current_players: u8,
	max_players: u8,
	bots: u8,
	server_type: ServerType,
	os_type: OsType,
	visibility: Visibility,
	vac_enabled: bool,
	game_version: String,
	extra_data: Vec<A2SExtraData>
}

impl A2SInfo {
	pub fn from_bytes(buf: &[u8]) -> anyhow::Result<Self> {
		if buf[0..=4] != *b"\xFF\xFF\xFF\xFF\x49" {
			return Err(anyhow::anyhow!("Invalid A2S_INFO response header"));
		}
		let protocol = buf[5];
		let mut cursor = 6;
		let mut server_name = String::new();
		loop {
			let byte = buf[cursor];
			if byte == b'\x00' {
				cursor += 1;
				break
			}
			server_name.push(byte as char);
			cursor += 1;
		}

		let mut map_name = String::new();
		loop {
			let byte = buf[cursor];
			if byte == b'\x00' {
				cursor += 1;
				break
			}
			map_name.push(byte as char);
			cursor += 1;
		}

		let mut game_dir = String::new();
		loop {
			let byte = buf[cursor];
			if byte == b'\x00' {
				cursor += 1;
				break
			}
			game_dir.push(byte as char);
			cursor += 1;
		}

		let mut game_name = String::new();
		loop {
			let byte = buf[cursor];
			if byte == b'\x00' {
				cursor += 1;
				break
			}
			game_name.push(byte as char);
			cursor += 1;
		}

		let app_id = u16::from_le_bytes([buf[cursor], buf[cursor + 1]]);
		cursor += 2;

		let current_players = buf[cursor];
		cursor += 1;

		let max_players = buf[cursor];
		cursor += 1;

		let bots = buf[cursor];
		cursor += 1;

		let server_type_char = buf[cursor];
		let server_type = match server_type_char {
			b'd' => ServerType::Dedicated,
			b'l' => ServerType::Listen,
			b'p' => ServerType::SourceTV,
			_ => return Err(anyhow::anyhow!("Unknown server type: {}", server_type_char)),
		};
		cursor += 1;

		let env_char = buf[cursor];
		let os_type = match env_char {
			b'l' => OsType::Linux,
			b'w' => OsType::Windows,
			b'm' | b'o' => OsType::Mac,
			_ => return Err(anyhow::anyhow!("Unknown OS type: {}", env_char)),
		};
		cursor += 1;

		let visibility = match buf[cursor] {
			0 => Visibility::Public,  // Public
			1 => Visibility::Private, // Private
			_ => return Err(anyhow::anyhow!("Unknown visibility type: {}", buf[cursor])),
		};
		cursor += 1;

		let vac_enabled = buf[cursor] == 1;
		cursor += 1;

		let mut game_version = String::new();
		loop {
			let byte = buf[cursor];
			if byte == b'\x00' {
				cursor += 1;
				break
			}
			game_version.push(byte as char);
			cursor += 1;
		}
		let mut extra_data = vec![];
		if let Some(edf) = buf.get(cursor) {
			cursor += 1;
			extra_data = A2SExtraData::from_bytes(*edf, &buf[cursor..])?;
		}

		Ok(Self {
			protocol_ver: protocol,
			server_name,
			map_name,
			game_dir,
			game_name,
			app_id,
			current_players,
			max_players,
			bots,
			server_type, // Assuming dedicated server for simplicity
			os_type,
			visibility,
			vac_enabled,
			game_version,
			extra_data, // Extra data parsing can be added later
		})
	}
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let args = Args::parse();

	let addr = format!("{}:{}",args.host, args.port);
	let addr: SocketAddr = addr.parse().context("Invalid host/port")?;

	let socket = UdpSocket::bind("0.0.0.0:0").await.context("Failed to bind UDP socket")?;

	let query = b"\xFF\xFF\xFF\xFFTSource Engine Query\x00";
	socket.send_to(query, &addr).await.context("Failed to send A2S_INFO request")?;

	let mut buf = [0u8; 1400];
	let (_,_) = socket.recv_from(&mut buf).await.context("Failed to receive response")?;

	let challenge = u32::from_le_bytes([buf[5], buf[6], buf[7], buf[8]]);
	let mut rebuttal = b"\xFF\xFF\xFF\xFFTSource Engine Query\x00".to_vec();
	rebuttal.extend(&challenge.to_le_bytes());

	socket.send_to(&rebuttal, &addr).await.context("Failed to send A2S_INFO request")?;
	let (len, _) = socket.recv_from(&mut buf).await.context("Failed to receive response")?;

	let a2s_info = A2SInfo::from_bytes(&buf[..len]).context("Failed to parse A2S_INFO response")?;
	let a2s_json = serde_json::to_string_pretty(&a2s_info).context("Failed to serialize A2S_INFO to JSON")?;
	println!("{a2s_json}");


	Ok(())
}
