#[path = "../domain/constants.rs"]
mod constants;

use std::io::{Read, Write};
use std::time::Instant;
use constants::TIME_OUT;

pub struct Access {
    pub channel: Option<ssh2::Channel>,
}

impl Access {
    pub fn channel_command(&mut self, command: &str) -> String {
        if let Some(ref mut channel) = self.channel {
            if let Err(e) = channel.write_all(command.as_bytes()) {
                eprintln!("Falha ao enviar comando: {}", e);
                return String::new();
            }
            if let Err(e) = channel.write_all(b"\n") {
                eprintln!("Falha ao enviar nova linha: {}", e);
                return String::new();
            }

            let mut output: String = String::new();
            let mut buffer: [u8; 1024] = [0; 1024];
            let start_time = Instant::now();

            loop {
                if start_time.elapsed() >= TIME_OUT {
                    break;
                }

                match channel.read(&mut buffer) {
                    Ok(0) => {
                        println!("EOF ou canal fechado");
                        break;
                    }
                    Ok(n) => {
                        output.push_str(&String::from_utf8_lossy(&buffer[..n]));
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(TIME_OUT);
                    }
                    Err(e) => {
                        eprintln!("Falha ao ler saída do comando: {}", e);
                        break;
                    }
                }
            }
            return output;
        } 
        
        String::new()
    
    }

    pub fn execute_commands(&mut self, commands: Vec<&str>) -> Vec<String> {
        let mut results = Vec::new();

        if let Some(ref mut channel) = self.channel {
            let _ = channel.shell();
            for command in commands {
                let output = self.channel_command(command);
                results.push(output);
                std::thread::sleep(TIME_OUT);
            }
        }
        results
    }
}
