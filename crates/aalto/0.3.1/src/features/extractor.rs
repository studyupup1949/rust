use std::collections::HashMap;

use image::{DynamicImage, GenericImageView};

use crate::{value::Value};

#[cfg(feature = "wasm")]
pub mod wasm;

pub fn load_local_image(img_path: String) -> Result<DynamicImage, image::ImageError> {
    // Load the image
    image::open(img_path)
}

fn invert_hashmap<K, V>(hashmap: &HashMap<K, V>) -> HashMap<&V, Vec<&K>> where V: std::hash::Hash, V: std::cmp::Eq {
    let mut new_hashmap: HashMap<&V, Vec<&K>> = HashMap::new();

    for (k, v) in hashmap {
        match new_hashmap.get_mut(&v) {
            Some( existing ) => {
                existing.push(k);
            },
            None => {
                new_hashmap.insert(v, vec![k]);
            },
        }
    }

    new_hashmap
}

pub fn extract_from_image(image: DynamicImage, n: usize) -> Vec<crate::value::Value> {
    let w = image.width();
    let h = image.height();
    println!("{:?}", (w, h));

    let mut tiles: HashMap<(u32, u32), String> = HashMap::new();
    let mut rules = vec![];

    for y in 0..h-(n as u32-1) {
        for x in 0..w-(n as u32-1) {
            let cropped = image.view(x, y, n as u32, n as u32);
            let b64 = image_to_base64(image::DynamicImage::ImageRgba8(cropped.to_image()));
            tiles.insert((x, y), b64);
        }
    }

    let tile_positions = invert_hashmap(&tiles);
    // println!("{}", tiles.keys().len());
    // println!("{:?}", tile_positions);

    let mut existing_connectors: HashMap<(String, String, usize), String> = HashMap::new();
    let mut latest_connector = 0;

    let mut index = 0;
    for (b64, known_positions) in tile_positions {
        let mut connectors = [
            vec![],
            vec![],
            vec![],
            vec![],
        ];

        for position in known_positions {
            let x = position.0;
            let y = position.1;
            
            let mut neighbours: [Option<&String>; 4] = [None, None, None, None];
            if y > n as u32 - 1 {
                neighbours[0] = tiles.get(&(x, y-n as u32)); // Up
            }
            if x < w - n as u32 {
                neighbours[3] = tiles.get(&(x+n as u32, y)); // Right
            }
            if y < h - n as u32 {
                neighbours[2] = tiles.get(&(x, y+n as u32)); // Down
            }
            if x > n as u32 - 1 {
                neighbours[1] = tiles.get(&(x-n as u32, y)); // Left
            }

            let mut n_index: usize = 0;
            for i in neighbours {
                let n_index_reversed = (n_index + 2) % 4;
                if let Some( n ) = i {
                    // println!("Bridging {} to {} from direction {}", b64, n, n_index);
                    // println!("{:?}, {:?}", position, neighbours);
                    let keys = (
                        (
                            b64.clone(),
                            n.clone(),
                            n_index
                        ),
                        (
                            n.clone(),
                            b64.clone(),
                            n_index_reversed
                        ),
                    );
                    let connector_value = match existing_connectors.get(&keys.0) {
                        Some( connector ) => {
                            connector.clone()
                        },
                        None => {
                            match existing_connectors.get(&keys.1) {
                                Some( connector ) => connector.clone(),
                                None => {
                                    let v = format!("{}", latest_connector);
                                    existing_connectors.insert(keys.0, v.clone());
                                    latest_connector += 1;

                                    v
                                },
                            }
                        },
                    };
                    if !connectors[n_index].contains(&connector_value) {
                        connectors[n_index].push(connector_value);
                    }
                }
                n_index += 1;
            }
        }

        let rule = Value {
            value: index.to_string(),
            color: b64.clone(),
            disallow: None,
            connectors
        };
        rules.push(rule);
        index += 1;
    }

    println!("{}", serde_json::to_string(&rules).unwrap());
    rules
}

pub fn image_to_base64(image: DynamicImage) -> String {
    let mut png: Vec<u8> = Vec::new();
    image.write_to(&mut std::io::Cursor::new(&mut png), image::ImageOutputFormat::Png)
        .expect("Failed to convert image to png");
    let res_base64 = base64::encode(&png);
    
    format!("url('data:image/png;base64,{}')", res_base64)
}

#[cfg(test)]
mod tests {
    #[test]
    fn image_loading_works() {
        let _ = super::load_local_image("tests/images/Flowers.png".to_string()).unwrap();
    }
    #[test]
    fn extraction_works_flowers() {
        let image = super::load_local_image("tests/images/Flowers.png".to_string()).unwrap();
        let _ = super::extract_from_image(image, 3);
    }
    #[test]
    fn extraction_works_cat() {
        let image = super::load_local_image("tests/images/Cat.png".to_string()).unwrap();
        let _ = super::extract_from_image(image, 3);
    }
}