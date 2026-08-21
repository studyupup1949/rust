use opengl_graphics::*;
use piston::event_loop::*;
use piston::input::*;
use piston::window::WindowSettings;
use sdl2_window::Sdl2Window;

use holomaze::*;

fn main() {
    let opengl = OpenGL::V3_2;
    let mut window: Sdl2Window = WindowSettings::new(
        "HoloMaze: Toy model of physical Seshatic identity", [512, 512])
        .exit_on_esc(true)
        .graphics_api(opengl)
        .build()
        .unwrap();

    let mut layout: Layout = Default::default();
    let bg_color = [0.3, 0.3, 0.3, 1.0];
    let cell_bg_color = [0.5, 0.5, 0.5, 1.0];
    let cell_player_color = [1.0, 1.0, 0.0, 1.0];
    let cell_follower_color = [0.2, 0.5, 0.0, 1.0];
    let cell_unknown_color = [0.0, 0.0, 0.0, 0.0];
    let cell_false_color = [1.0, 0.0, 0.0, 1.0];
    let cell_true_color = [0.0, 0.0, 1.0, 1.0];

    let games: Vec<Game> = vec![
        games::diagonal(),
        games::snake(),
        games::clock(),
        games::coil(),
    ];
    let mut game_ind = 0;

    let mut map: Map = Map::new(games[game_ind].f);
    (games[game_ind].config)(&mut map);
    use piston::AdvancedWindow;
    window.set_title(games[game_ind].name.clone());

    let mut cur: [f64; 2] = [0.0; 2];
    let mut gl = GlGraphics::new(opengl);
    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(pos) = e.mouse_cursor_args() {
            cur = pos;
        }
        if let Some(Button::Mouse(MouseButton::Left)) = e.press_args() {
            if let Some(pos) = layout.hit(cur) {
                let mut map2 = map.clone();
                if map2.mov(pos) {
                    map = map2;
                }
            }
        } else if let Some(Button::Keyboard(Key::R)) = e.press_args() {
            map = Map::new(map.f);
            (games[game_ind].config)(&mut map);
        } else if let Some(Button::Keyboard(Key::Up)) = e.press_args() {
            game_ind = (game_ind + 1) % games.len();
            map = Map::new(games[game_ind].f);
            (games[game_ind].config)(&mut map);
            window.set_title(games[game_ind].name.clone());
        } else if let Some(Button::Keyboard(Key::Down)) = e.press_args() {
            game_ind = (game_ind + games.len() - 1) % games.len();
            map = Map::new(games[game_ind].f);
            (games[game_ind].config)(&mut map);
            window.set_title(games[game_ind].name.clone());
        }
        if let Some(args) = e.render_args() {
            use graphics::*;
            use vecmath::*;

            layout.update(&mut window);
            let cell_size = layout.cell_size();
            gl.draw(args.viewport(), |c, g| {
                clear(bg_color, g);
                for j in 0..4 {
                    for i in 0..4 {
                        let pos = layout.cell_pos([i, j]);
                        let to = vec2_add(pos, cell_size);
                        
                        rectangle_from_to(
                            cell_bg_color,
                            pos,
                            to,
                            c.transform,
                            g
                        );
                        let color = match map.cells[j][i] {
                            Cell::Player => cell_player_color,
                            Cell::Follower => cell_follower_color,
                            Cell::Unknown => cell_unknown_color,
                            Cell::Val(false) => cell_false_color,
                            Cell::Val(true) => cell_true_color,
                        };
                        rectangle_from_to(
                            color,
                            vec2_add(pos, layout.cell_margin()),
                            vec2_sub(to, layout.cell_margin()),
                            c.transform,
                            g
                        );
                    }
                }
            });
        }
    }
}

pub struct Layout {
    pub offset: f64,
    pub wsize: [f64; 2],
}

impl Default for Layout {
    fn default() -> Layout {
        Layout {
            offset: 10.0,
            wsize: [512.0; 2],
        }
    }
}

impl Layout {
    pub fn update(&mut self, w: &impl piston::Window) {
        self.wsize = w.size().into();
    }

    pub fn top_left(&self) -> [f64; 2] {
        [self.offset; 2]
    }

    pub fn cell_margin(&self) -> [f64; 2] {
        [self.offset; 2]
    }

    pub fn cell_size(&self) -> [f64; 2] {
        use vecmath::*;
        vec2_scale(
            vec2_sub(self.wsize, [(2.0 + 3.0) * self.offset; 2]),
            1.0 / 4.0
        )
    }

    pub fn cell_pos(&self, [i, j]: [usize; 2]) -> [f64; 2] {
        use vecmath::*;
        vec2_add(self.top_left(), vec2_mul(
            vec2_add(self.cell_size(), self.cell_margin()),
            [i as f64, j as f64]))
    }

    pub fn hit(&self, pos: [f64; 2]) -> Option<[usize; 2]> {
        use vecmath::*;
        
        let cell_size = vec2_add(self.cell_size(), self.cell_margin());
        let inv_cell_size = [1.0 / cell_size[0], 1.0 / cell_size[1]];
        let p = vec2_mul(vec2_sub(pos, [self.offset; 2]), inv_cell_size);
        if p[0] <= 0.0 || p[0] >= 5.0 {return None}
        if p[1] <= 0.0 || p[1] >= 5.0 {return None}
        Some([p[0] as usize, p[1] as usize])
    }
}

