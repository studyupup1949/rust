# Ad Event Log Parser

## Project Description
`ad_event_log_parser` This project parses ad event logs and extracts relevant details such as date, time, UUID, IP address, and price from log entries. The log format is defined as:
`"date time UUID IP price"`.

For example "2024-07-20 12:34:56 123e4567-e89b-12d3-a456-426614174000 192.168.1.1 19.99"


### Parsing Process

The log data is parsed using the `pest` parser, which is configured with the following grammar rules:

- **Date (`YYYY-MM-DD`)**: The date is parsed from the log, expected in the format `2023-07-20`.
- **Time (`HH:MM:SS`)**: The time is parsed from the log in the format `12:34:56`.
- **UUID (`XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX`)**: The UUID is parsed as a string of hexadecimal digits separated by hyphens.
- **IP (`XXX.XXX.XXX.XXX`)**: The IP address is expected to be in the form of `192.168.1.1`.
- **Price (`$XX.XX`)**: The price is parsed as a number prefixed with a dollar sign(optionally).

The results of the parsing will be used to extract structured data from the log, which can then be processed or saved in formats such as CSV.

### Grammar

The grammar used for parsing is defined in the `grammar.pest` file. The relevant rules include:

```pest
// Date: Matches the format YYYY-MM-DD
date = @{ DIGIT{4} ~ "-" ~ DIGIT{2} ~ "-" ~ DIGIT{2} }

// Time: Matches the format HH:MM:SS
time = @{ DIGIT{2} ~ ":" ~ DIGIT{2} ~ ":" ~ DIGIT{2} }

// UUID: Matches the format XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX
uuid = @{ HEX{8} ~ "-" ~ HEX{4} ~ "-" ~ HEX{4} ~ "-" ~ HEX{4} ~ "-" ~ HEX{12} }

// IP: Matches the format XXX.XXX.XXX.XXX
ip = @{ DIGIT{1,3} ~ "." ~ DIGIT{1,3} ~ "." ~ DIGIT{1,3} ~ "." ~ DIGIT{1,3} }

// Price: Matches the format $XX.XX
price = @{ "$"? ~ DIGIT+ ~ ("." ~ DIGIT{2})? }


A diagram representing the structure of a log entry can help visualize this:

+------------+------------+--------------------------------------+-----------------+--------+
|    Date    |    Time    |              UUID                    |       IP        | Price  |
+------------+------------+--------------------------------------+-----------------+--------+
| 2023-07-20 | 12:34:56   | 123e4567-e89b-12d3-a456-426614174000 | 192.168.1.1     | $19.99 |
+------------+------------+--------------------------------------+-----------------+--------+
```
## How to Use
You can run the parser with the following command:
    
    cargo run -- "<log_data>"
Where `<log_data>` is the log string to be parsed. For example:
    
    cargo run -- "2023-07-20 12:34:56 123e4567-e89b-12d3-a456-426614174000 192.168.1.1 $19.99"

This will output the parsed data in CSV format. Alternatively, you can use the --help option to get help information.

