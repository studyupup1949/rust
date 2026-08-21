use std::error::Error;
use std::io;

use a8mini_camera_rs::control::A8MiniSimpleCommand;
use a8mini_camera_rs::A8Mini;


fn print_ascii_command_table() {
  let commands = [
    "AutoCenter", "RotateUp", "RotateDown", "RotateRight", "RotateLeft", "StopRotation", "ZoomIn", "ZoomOut", "ZoomMax",
    "MaxZoomInformation", "FocusIn", "FocusOut", "TakePicture", "RecordVideo", "Rotate100100", "CameraInformation",
    "AutoFocus", "HardwareIDInformation", "FirmwareVersionInformation", "SetLockMode", "SetFollowMode", "SetFPVMode",
    "AttitudeInformation", "SetVideoOutputHDMI", "SetVideoOutputCVBS", "SetVideoOutputOff", "LaserRangefinderInformation", 
    "RebootCamera", "RebootGimbal",
  ];

  let header = "+----+------------------------------+";
  println!("{}", header);
  println!("| ID | Command Name                 |");
  println!("{}", header);

  for (i, command) in commands.iter().enumerate() {
    println!("| {:>2} | {:<28} |", i, command);
  }

  println!("{}", header);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  print_ascii_command_table();
  
  loop {
    let command: &str;
    println!("Awaiting command:");
    let stdin = io::stdin();
    let buf = &mut String::new();
    stdin.read_line(buf)?;
    command = buf.strip_suffix("\n").unwrap();

    let command_enum: Option<A8MiniSimpleCommand> = match command {
      "0"   | "AutoCenter" => Some(A8MiniSimpleCommand::AutoCenter),
      "1"   | "RotateUp" => Some(A8MiniSimpleCommand::RotateUp),
      "2"   | "RotateDown" => Some(A8MiniSimpleCommand::RotateDown),
      "3"   | "RotateRight" => Some(A8MiniSimpleCommand::RotateRight),
      "4"   | "RotateLeft" => Some(A8MiniSimpleCommand::RotateLeft),
      "5"   | "StopRotation" => Some(A8MiniSimpleCommand::StopRotation),
      "6"   | "ZoomIn" => Some(A8MiniSimpleCommand::ZoomIn),
      "7"   | "ZoomOut" => Some(A8MiniSimpleCommand::ZoomOut),
      "8"   | "ZoomMax" => Some(A8MiniSimpleCommand::ZoomMax),
      "9"   | "MaxZoomInformation" => Some(A8MiniSimpleCommand::MaxZoomInformation),
      "10"  | "FocusIn" => Some(A8MiniSimpleCommand::FocusIn),
      "11"  | "FocusOut" => Some(A8MiniSimpleCommand::FocusOut),
      "12"  | "TakePicture" => Some(A8MiniSimpleCommand::TakePicture),
      "13"  | "RecordVideo" => Some(A8MiniSimpleCommand::RecordVideo),
      "14"  | "Rotate100100" => Some(A8MiniSimpleCommand::Rotate100100),
      "15"  | "CameraInformation" => Some(A8MiniSimpleCommand::CameraInformation),
      "16"  | "AutoFocus" => Some(A8MiniSimpleCommand::AutoFocus),
      "17"  | "HardwareIDInformation" => Some(A8MiniSimpleCommand::HardwareIDInformation),
      "18"  | "FirmwareVersionInformation" => Some(A8MiniSimpleCommand::FirmwareVersionInformation),
      "19"  | "SetLockMode" => Some(A8MiniSimpleCommand::SetLockMode),
      "20"  | "SetFollowMode" => Some(A8MiniSimpleCommand::SetFollowMode),
      "21"  | "SetFPVMode" => Some(A8MiniSimpleCommand::SetFPVMode),
      "22"  | "AttitudeInformation" => Some(A8MiniSimpleCommand::AttitudeInformation),
      "23"  | "SetVideoOutputHDMI" => Some(A8MiniSimpleCommand::SetVideoOutputHDMI),
      "24"  | "SetVideoOutputCVBS" => Some(A8MiniSimpleCommand::SetVideoOutputCVBS),
      "25"  | "SetVideoOutputOff" => Some(A8MiniSimpleCommand::SetVideoOutputOff),
      "26"  | "LaserRangefinderInformation" => Some(A8MiniSimpleCommand::LaserRangefinderInformation),
      "27"  | "RebootCamera" => Some(A8MiniSimpleCommand::RebootCamera),
      "28"  | "RebootGimbal" => Some(A8MiniSimpleCommand::RebootGimbal),
      _ => None,
    };

    if command_enum.is_some() {
      println!("Sending {:?}", command_enum);
      let camera: A8Mini = A8Mini::connect().await?;
      camera.send_command_blind(command_enum.unwrap()).await?;
    }
    else {
      break;
    }
  }

  Ok(())
}

