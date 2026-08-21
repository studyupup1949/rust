#Rust Project
This project consists of different CLI tools that used Rust crates to provide a variety of functionalities.

Below are the tools l developed:

Commands:
  grep <pattern> <file>                     Search for a pattern in a file
  cat <file1> [file2...]                    Concatenate and display file contents
  ls <directory>                            List directory contents
  diff <file1> <file2>                      Compare two files
  time                                      Display current time
  date                                      Display current date
  calc                                      Run a calculator
  password                                  Generate a random password
  currencyconvert <amount> <from> <to>      Convert currency
  weather <location>                        Fetch weather information
  shorten <long_url>                        Shorten a URL
  define <word>                             Look up word definition
  reverse <string>                          Reverse a string
  analyze <data>                            Analyze text data
  xo                                        Play Tic-Tac-Toe game
  

  Usage:
  cargo run -- <command> eg for weather:
  cargo run -- weather "New York"