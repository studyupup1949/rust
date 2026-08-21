#[path = "../functions/clear_terminal.rs"] mod clear_termianl;
#[path = "../functions/read_float_from_keyboard.rs"] mod read_float_from_keyboard;
#[path = "../functions/ping.rs"] mod ping;
mod access;
mod constants;

use crate::access::Access;
use clear_termianl::clear_terminal;
use read_float_from_keyboard::read_float_from_keyboard;

pub struct Adjustment {
    pub ip: String,
    pub channel_conf: Access,
}

impl Adjustment {
    pub fn reboot(&mut self) {
        let commands: Vec<&str> = vec!["rtkbosa -b /etc/config/rtkbosa_k.bin", "reboot"];
        self.channel_conf.execute_commands(commands);
    }

    pub fn get_value_register(&mut self) -> i32 {
        let commands: Vec<&str> = vec![
            "diag",
            "i2c bosa_calibrate get dev 0x51 reg 0xa8 count 1",
            "exit",
        ];
        let result_vec: Vec<String> = self.channel_conf.execute_commands(commands);
        println!("{}", result_vec[2]);
        let hex_string: i32 = i32::from_str_radix(
            &result_vec[2]
                .trim()
                .replace(" ", "")
                .replace("RTK.0>", "")
                .replace("i2cbosa_calibrategetdev0x51reg0xa8count1", "")
                .replace("\n", ""),
            16,
        )
        .expect("Falha ao converter número hexadecimal para inteiro");
        hex_string
    }

    pub fn set_value_register(&mut self, value: i32) {
        let command = format!(
            "i2c bosa_calibrate set dev 0x51 reg 0xa8 count 1 data {:X}",
            value
        );
        let commands: Vec<&str> = vec!["diag", &command, "exit"];
        self.channel_conf.execute_commands(commands);
    }

    pub fn check_value(&mut self) {
        let mut measured_value: f64 = read_float_from_keyboard();

        loop {
            if measured_value > 2.2 {
                loop {
                    let register_value: i32 = self.get_value_register();
                    if measured_value > 2.2 {
                        self.set_value_register(register_value - 4);
                    } else if measured_value < 2.0 {
                        self.set_value_register(register_value + 1);
                    } else {
                        break;
                    }
                    measured_value = read_float_from_keyboard();
                }
            } else if measured_value < 2.0 {
                loop {
                    let register_value: i32 = self.get_value_register();
                    if measured_value > 2.2 {
                        self.set_value_register(register_value - 1);
                    } else if measured_value < 2.0 {
                        self.set_value_register(register_value + 4);
                    } else {
                        break;
                    }
                    measured_value = read_float_from_keyboard();
                }
            } else {
                break;
            }
        }
    }

    pub fn finalize_adjustment(&mut self) {
        clear_terminal();
        println!("Valor medido está na faixa!");
        println!("Aguarde reiniciar a onu!");
        println!("Reiniciando");
        self.reboot();
        std::thread::sleep(core::time::Duration::from_secs(5)); 
        loop {
            match ping::ping(&constants::IP.to_string()) {
                Ok(_output) => {
                    println!("Desconecte a Onu!");
                    std::thread::sleep(core::time::Duration::from_secs(10));
                    clear_terminal();
                    break;
                }
                Err(_e) => {
                    eprintln!("Aguarde...");
                    std::thread::sleep(constants::TIME_OUT);
                }
            }
        }
    }
}
