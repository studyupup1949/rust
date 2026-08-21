/*!
 Act is a simple engine for making simple, text-based adventure games.
 It's on [crates.io](https://crates.io/crates/act) and on [GitHub](https://github.com/ichy-wayland/act)!
 # Examples
 ```
extern crate act;

 use act::load_game;

 fn main() {
     // Create a string containing our Act game
     let game_string = r#"
     {
        "rooms": [
            {
                "name": "start",
                "scene": "Im a starting room! Welcome to this example game.",
                "actions": [
                    {
                        "variant": "Move",
                        "fields": [
                            "Move to another room","example",""
                        ]
                    }
                ]
            },
            {
                "name": "example",
                "scene": "You enter an example room, with a big, triangular key in it. Theres also a door with a keyhole in triangular shape.",
                "actions": [
                    {
                        "variant": "PickUp",
                        "fields": [
                            "Pick the key up","TriangleKey",""
                        ]
                    },
                    {
                        "variant": "Move",
                        "fields": [
                            "Try to open the door","locked","TriangleKey"
                        ]
                    }
                ]
            },
            {
                "name": "locked",
                "scene": "You picked an item up and used it to open the door! This is the final room. Congratz!",
                "actions": [
                    {
                        "variant": "Move",
                        "fields": [
                            "Move to another room","example",""
                        ]
                    }
                ]
            }
        ]
     }
     "#;
     // Load the game into a proper Game struct
     let mut game = load_game(game_string).unwrap();
     // Start the game
     game.play();
     // Profit!
 }
 ```
*/

extern crate rustc_serialize;
extern crate ansi_escapes;

use std::io;
use std::collections::HashMap;
use std::{thread, time};

use rustc_serialize::json;


macro_rules! try_opt_res {
    ($o:expr) => {
        match $o {
            Ok(x) => x,
            Err(_) => return None
        }
    }
}

///
///An Action is composed of three strings.
///The first one is the text that will be shown to the user.
///The second is what the action will do.  For example, in a PickUp action, it would be the item that would be given to the user.
///The third one is the requirement. This will check if the user  has the item specified, and only if true will proceed.
///# Examples
///```
///use act::Action;
///
///fn main() {
///    let move_to_locked = Action::Move("Unlock the door ","locked_room","LockedRoomKey");
///}
///```
///
#[derive(Clone,Debug)]
pub enum Action {
    PickUp(String,String,String),
    Move(String,String,String)
}

impl Action {
    pub fn text(&self) -> String {
        match &self {
            &&Action::PickUp(ref s,_,_) => s.clone(),
            &&Action::Move(ref s,_,_) => s.clone()
        }
    }
}

#[derive(Clone)]
pub struct Room {
    pub scene: String,
    pub actions: Vec<Action>
}

pub struct Character {
    inventory: HashMap<String,String>,
    room: String
}

pub struct Game {
    rooms: HashMap<String,Room>,
    character: Character
}

impl Game {
    pub fn action(&mut self, a: Action) {
        match a {
            Action::PickUp(_,i,r) => {
                if self.character.inventory.contains_key(&r) || &r == "" {
                    self.character.inventory.insert(i.clone(),i);
                }
            },
            Action::Move(_,r,i) => {
                if self.character.inventory.contains_key(&i) || &i == "" {
                    self.character.room = r;
                    self.clear();
                } else {
                    println!("For opening this room, you need to have a {}",i);
                }
            }
        }
    }

    pub fn render_room(&mut self,r: Room) {
        for c in r.scene.lines() {
            println!("{}",c);
        }
        println!("");
        for (i,a) in r.actions.iter().enumerate() {
            println!("{}. {}\n",i,a.text());
        }
        println!("");
    }

    pub fn clear(&self) {
        print!("{}",ansi_escapes::ClearScreen);
    }

    pub fn play(&mut self) {

        println!("Made with \n");
        print!("
      _/           _//   _/// _//////
     _/ //      _//   _//     _//
    _/  _//    _//            _//
   _//   _//   _//            _//
  _////// _//  _//            _//
 _//       _//  _//   _//     _//
_//         _//   _////       _//
\n");
        println!("Make your own game at github.com/ichy-wayland/act");
        thread::sleep(time::Duration::from_millis(4000));
        self.clear();
        'outer: loop {
            let rooms = self.rooms.clone();
            let r = rooms.get(&self.character.room).unwrap();
            self.render_room(r.clone());

            let mut s = String::new();
            io::stdin().read_line(&mut s).unwrap();

            match s.chars().nth(0).unwrap().to_digit(10) {
                Some(u) => {
                    let a = match r.actions.iter().nth(u as usize) {
                        Some(x) => x,
                        None => { continue 'outer; }
                    };
                    self.action(a.clone());
        //            self.character.on_action(a);
                },
                None => { continue 'outer }
            };
        }
    }
}

pub fn load_game(s: &str) -> Result<Game,json::DecoderError> {
    use raw::*;
    let rg: RawGame = try!(json::decode(&s));
    Ok(rg.process())
}

mod raw {
    use std::collections::HashMap;
    use super::*;

    #[derive(Clone,RustcDecodable)]
    pub struct RawGame {
        rooms: Vec<RawRoom>
    }

    impl RawGame {
        pub fn process(&self) -> Game {
            use std::io;

            let mut r_hash: HashMap<String,Room> = HashMap::new();
            for r in self.rooms.clone() {
                r_hash.insert(r.name.clone(),Room {
                    scene: r.scene.clone(),
                    actions: r.process_actions()
                });
            }
            let character = Character {
                inventory: HashMap::new(),
                room: "start".into()
            };
            Game {
            //    out: io::stdout(),
                rooms: r_hash,
                character: character
            }
        }
    }

    #[derive(Clone,RustcDecodable)]
    pub struct RawAction {
        pub variant: String,
        fields: Vec<String>
    }

    impl RawAction {
        pub fn process(&self) -> Action {
            match &*self.variant {
                "PickUp" => {
                    Action::PickUp(self.fields[0].clone(),self.fields[1].clone(),self.fields[2].clone())
                },
                "Move" => { Action::Move(self.fields[0].clone(),self.fields[1].clone(),self.fields[2].clone()) },
                _ => { panic!("Could not parse Action!") }
            }
        }
    }
    #[derive(Clone,RustcDecodable)]
    pub struct RawRoom {
        pub name: String,
        pub scene: String,
        pub actions: Vec<RawAction>
    }

    impl RawRoom {
        pub fn process_actions(&self) -> Vec<Action> {
            let mut actions: Vec<Action> = Vec::new();
            for a in self.actions.clone() {
                actions.push(a.process())
            }
            actions
        }
    }
}
