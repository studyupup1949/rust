use crate::value::Value;

pub fn checkers() -> Vec<Value> {
    vec![
        Value {
            value: String::from("w"),
            color: String::from("white"),
            disallow: None,
            connectors: [
                vec![String::from("1")],
                vec![String::from("2")],
                vec![String::from("3")],
                vec![String::from("4")],
            ]
        },Value {
            value: String::from("b"),
            color: String::from("black"),
            disallow: None,
            connectors: [
                vec![String::from("3")],
                vec![String::from("4")],
                vec![String::from("1")],
                vec![String::from("2")],
            ]
        },
    ]
}

pub fn terrain() -> Vec<Value> {
    vec![
        Value {
            value: String::from("."),
            color: String::from("darkblue"),
            disallow: None,
            connectors: [
                vec![String::from("dblue")],
                vec![String::from("dblue")],
                vec![String::from("dblue")],
                vec![String::from("dblue")],
            ]
        },
        Value {
            value: String::from("."),
            color: String::from("darkblue"),
            disallow: None,
            connectors: [
                vec![String::from("blue"), String::from("dblue")],
                vec![String::from("blue"), String::from("dblue")],
                vec![String::from("blue"), String::from("dblue")],
                vec![String::from("blue"), String::from("dblue")],
            ]
        },
        Value {
            value: String::from(":"),
            color: String::from("blue"),
            disallow: None,
            connectors: [
                vec![String::from("blue"), String::from("yellow")],
                vec![String::from("blue"), String::from("yellow")],
                vec![String::from("blue"), String::from("yellow")],
                vec![String::from("blue"), String::from("yellow")],
            ]
        },
        Value {
            value: String::from("o"),
            color: String::from("cornsilk"),
            disallow: None,
            connectors: [
                vec![String::from("yellow"), String::from("green")],
                vec![String::from("yellow"), String::from("green")],
                vec![String::from("yellow"), String::from("green")],
                vec![String::from("yellow"), String::from("green")],
            ]
        },
        Value {
            value: String::from("O"),
            color: String::from("green"),
            disallow: None,
            connectors: [
                vec![String::from("green")],
                vec![String::from("green")],
                vec![String::from("green")],
                vec![String::from("green")],
            ]
        },
    ]
}

pub fn dungeon() -> Vec<Value> {
    vec![
        Value {
          value: "█".to_string(), // Wall in
          color: "url('/aaltoweb/kenney/Tiles/tile_0000.png')".to_string(),
          disallow: None,
          connectors: [
            vec![
              "wall_in".to_string(),
              "wall_outU".to_string()
            ],
            vec![
              "wall_in".to_string()
            ],
            vec![
              "wall_in".to_string()
            ],
            vec![
              "wall_in".to_string()
            ]
          ]
        },
        Value {
          value: "░".to_string(), // Ground
          color: "url('/aaltoweb/kenney/Tiles/tile_0048.png')".to_string(),
          disallow: None,
          connectors: [
            vec![
              "wall_out".to_string()
            ],
            vec![
              "wall_out".to_string()
            ],
            vec![
              "wall_out".to_string()
            ],
            vec![
              "wall_out".to_string()
            ]
          ]
        },
        Value {
          value: "▓".to_string(), // Wall in variant
          color: "url('/aaltoweb/kenney/Tiles/tile_0012.png')".to_string(),
          disallow: None,
          connectors: [
            vec![
              "wall_in".to_string()
            ],
            vec![
              "wall_in".to_string()
            ],
            vec![
              "wall_in".to_string()
            ],
            vec![
              "wall_in".to_string()
            ]
          ]
        },
        Value {
          value: "▐".to_string(), // Wall side L
          color: "url('/aaltoweb/kenney/Tiles/tile_0015.png')".to_string(),
          disallow: None,
          connectors: [
            vec![
              "wall_sideL".to_string()
            ],
            vec![
              "wall_out".to_string()
            ],
            vec![
              "wall_sideL".to_string()
            ],
            vec![
              "wall_in".to_string()
            ]
          ]
        },
        Value {
          value: "▌".to_string(), // Wall side R
          color: "url('/aaltoweb/kenney/Tiles/tile_0013.png')".to_string(),
          disallow: None,
          connectors: [
            vec![
              "wall_sideR".to_string()
            ],
            vec![
              "wall_in".to_string()
            ],
            vec![
              "wall_sideR".to_string()
            ],
            vec![
              "wall_shadowR".to_string()
            ]
          ]
        },
        Value {
          value: "▐".to_string(), // Wall corner LD
          color: "url('/aaltoweb/kenney/Tiles/tile_0016.png')".to_string(),
          disallow: None,
          connectors: [
            vec![
              "wall_sideL".to_string()
            ],
            vec![
              "wall_out".to_string()
            ],
            vec![
              "wall_outLD".to_string()
            ],
            vec![
              "wall_sideD".to_string()
            ]
          ]
        },
        Value {
          value: "▇".to_string(), // Wall side D
          color: "url('/aaltoweb/kenney/Tiles/tile_0002.png')".to_string(),
          disallow: None,
          connectors: [
            vec![
              "wall_in".to_string()
            ],
            vec![
              "wall_sideD".to_string()
            ],
            vec![
              "wall_outD".to_string()
            ],
            vec![
              "wall_sideD".to_string()
            ]
          ]
        },
        Value {
          value: "▌".to_string(), // Wall corner RD
          color: "url('/aaltoweb/kenney/Tiles/tile_0017.png')".to_string(),
          disallow: None,
          connectors: [
            vec![
              "wall_sideR".to_string()
            ],
            vec![
              "wall_sideD".to_string()
            ],
            vec![
              "wall_outRD".to_string()
            ],
            vec![
              "wall_shadowR".to_string()
            ]
          ]
        },
        Value {
          value: "▀".to_string(), // Wall down
          color: "url('/aaltoweb/kenney/Tiles/tile_0040.png')".to_string(),
          disallow: None,
          connectors: [
            vec![
              "wall_outD".to_string()
            ],
            vec![
              "wall_down".to_string()
            ],
            vec![
              "wall_shadowD".to_string()
            ],
            vec![
              "wall_down".to_string()
            ]
          ]
        },
        Value {
          value: "▘".to_string(), // Wall down end right
          color: "url('/aaltoweb/kenney/Tiles/tile_0059.png')".to_string(),
          disallow: None,
          connectors: [
            vec![
              "wall_outRD".to_string()
            ],
            vec![
              "wall_down".to_string()
            ],
            vec![
              "wall_shadowD".to_string()
            ],
            vec![
              "wall_shadowR".to_string()
            ]
          ]
        },
        Value {
          value: "▝".to_string(), // Wall down end left
          color: "url('/aaltoweb/kenney/Tiles/tile_0057.png')".to_string(),
          disallow: None,
          connectors: [
            vec![
              "wall_outLD".to_string()
            ],
            vec![
              "wall_out".to_string()
            ],
            vec![
              "wall_shadowD".to_string()
            ],
            vec![
              "wall_down".to_string()
            ]
          ]
        },
        Value {
          value: "▄".to_string(), // Wall up
          color: "url('/aaltoweb/kenney/Tiles/tile_0026.png')".to_string(),
          disallow: None,
          connectors: [
            vec![
              "wall_out".to_string()
            ],
            vec![
              "wall_up".to_string()
            ],
            vec![
              "wall_outU".to_string()
            ],
            vec![
              "wall_up".to_string()
            ]
          ]
        },
        Value {
          value: "▖".to_string(), // Wall up end R
          color: "url('/aaltoweb/kenney/Tiles/tile_0005.png')".to_string(),
          disallow: None,
          connectors: [
            vec![
              "wall_out".to_string()
            ],
            vec![
              "wall_up".to_string()
            ],
            vec![
              "wall_sideR".to_string()
            ],
            vec![
              "wall_shadowUR".to_string()
            ]
          ]
        },
        Value {
          value: "▗".to_string(), // Wall up end L
          color: "url('/aaltoweb/kenney/Tiles/tile_0004.png')".to_string(),
          disallow: None,
          connectors: [
            vec![
              "wall_out".to_string()
            ],
            vec![
              "wall_out".to_string()
            ],
            vec![
              "wall_sideL".to_string()
            ],
            vec![
              "wall_up".to_string()
            ]
          ]
        },
        Value {
          value: "░".to_string(), // Wall shadow down
          color: "url('/aaltoweb/kenney/Tiles/tile_0050.png')".to_string(),
          disallow: None,
          connectors: [
            vec![
              "wall_shadowD".to_string()
            ],
            vec![
              "wall_out".to_string()
            ],
            vec![
              "wall_out".to_string()
            ],
            vec![
              "wall_out".to_string()
            ]
          ]
        },
        Value {
          value: "░".to_string(), // Wall shadow right
          color: "url('/aaltoweb/kenney/Tiles/tile_0050R.png')".to_string(),
          disallow: None,
          connectors: [
            vec![
              "wall_out".to_string()
            ],
            vec![
              "wall_shadowR".to_string()
            ],
            vec![
              "wall_out".to_string()
            ],
            vec![
              "wall_out".to_string()
            ]
          ]
        },
        Value {
          value: "░".to_string(), // Wall shadow UR
          color: "url('/aaltoweb/kenney/Tiles/tile_0053UR.png')".to_string(),
          disallow: None,
          connectors: [
            vec![
              "wall_out".to_string()
            ],
            vec![
              "wall_shadowUR".to_string()
            ],
            vec![
              "wall_shadowR".to_string()
            ],
            vec![
              "wall_out".to_string()
            ]
          ]
        }
    ]
}