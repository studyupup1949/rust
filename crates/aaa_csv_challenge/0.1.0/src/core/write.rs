use crate::err::Error;
use std::ops::Index;

pub fn replace_column(data: String, column: &str, replacement: &str) -> Result<String, Error> {
    let mut lines = data.lines();
    let headers = lines.next().unwrap();
    let columns: Vec<&str> = headers.split(',').collect(); //split(',').into_collection().to_vec(); //
    let column_number = columns.iter().position(|&e| e == column); //columns.index(column); //
    let column_number = match column_number {
        Some(clm) => clm,
        None => Err("column name dosen't exist in the input file")?,
    };

    let mut result = String::with_capacity(data.capacity());
    result.push_str(&columns.join(","));
    result.push('\n');

    for line in lines {
        let mut records: Vec<&str> = line.split(',').collect(); //split(',').into_collection().to_vec();//
        records[column_number] = replacement;
        result.push_str(&records.join(","));
        result.push('\n');
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::core::write::replace_column;

    #[test]
    fn test_replace_column() {
        let lines = r"First Name,Last Name,Age,City,Eyes color,Species
John,Doe,32,Tokyo,Blue,Human
Flip,Helm,12,Canberra,Red,Unknown
Terdos,Bendarian,165,Cracow,Blue,Magic tree
Dominik,Elpos,33,Paris,Purple,Orc
Brad,Doe,42,Dublin,Blue,Human
Ewan,Grath,51,New Delhi,Green,Human";
        let opn = replace_column(lines.to_string(), "City", "Beijing");
        println!("####, opn = {:?}", &opn);
        println!("####, opn.unwrap() = {:#?}", &opn.unwrap());
        // println!("####, opn.unwrap() = {:#?}", &opn.unwrap());
    }
}
