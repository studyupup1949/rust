# acro-rs
This was done to help me query a csv file with acronyms and their meanings. But I guess it can be used for any csv file that has two columns of which you want to use one to get the other.

## Installation
### With cargo
```cargo install acro```
### With homebrew
```
brew tap nilventosa/acro
brew install acro
```

## Usage
```acro [OPTIONS] <acronym>```

Arguments:
  - ```<acronym>```  the acronym to query

Options (short, long, env variable):
-  **-f, --file, ACRO_FILE \<file>**  
_the csv file with the acronyms and definitions_  
-  **-a, --acro, ACRO_COLUMN <acro_column>**             
_the column with the acronyms [default: 1]_  
-  **-d, --definition, DEFINITION_COLUMN <definition_column>**  
_the column with the definitions [default: 2]_  
-  **-D, --delimiter, ACRO_DELIMITER \<delimiter>**           
_delimiter character between columns [default: ',']_  
-  **-H, --header, ACRO_HEADER**                          
_flag if there is a header line_  
-  **-c, --color, ACRO_COLOR**                           
_disables color output_  
-  **-h, --help**                            
_Print help information_  
-  **-V, --version**                         
_Print version information_  
