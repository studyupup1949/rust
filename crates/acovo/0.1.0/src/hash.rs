#[cfg(feature = "hash")]
pub fn crc64_str(data:&str,seperator:Option<&str>)->String {
    //use crc64::crc64;
    let cksum = crc64::crc64(0, data.as_bytes()).to_be_bytes();
    let s = hex::encode(cksum).to_uppercase();
    if seperator.is_some() {
        let chars: Vec<char> = s.chars().collect();
        let split = &chars.chunks(4)
            .map(|chunk| chunk.iter().collect::<String>())
            .collect::<Vec<_>>();
        split.join(seperator.unwrap()).to_string()
    }else {
        s
    }
}

#[cfg(test)]
#[cfg(feature = "hash")]
mod tests {
    use super::*;

    #[test]
    fn test_crc64_str() {
        let result = crc64_str("1234567890",Some("-"));
        println!("test_crc64_str: {}",result);
        assert_eq!(result.len()>0, true);
    }
}