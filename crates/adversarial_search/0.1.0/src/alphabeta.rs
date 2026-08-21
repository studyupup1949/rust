use crate::utils::max;
use crate::utils::min;

pub fn alphabeta<N, FN, IN, FT, FE>(
    node: &N,
    depth: i32,
    successors: &FN,
    terminal: &FT,
    evaluation: &FE,
    is_maximizing_player: bool,
    mut alpha: f32,
    mut beta: f32,
) -> f32
where
    FN: Fn(&N) -> IN,
    IN: IntoIterator<Item = N>,
    FT: Fn(&N) -> Option<f32>,
    FE: Fn(&N) -> i32,
{
    let terminality = terminal(&node);
    if terminality.is_some() {
        return terminality.unwrap();
    }
    let mut val = match is_maximizing_player {
        true => -std::f32::INFINITY,
        false => std::f32::INFINITY,
    };
    for child in successors(&node) {
        let child_val = alphabeta(
            &child,
            depth - 1,
            successors,
            terminal,
            evaluation,
            !is_maximizing_player,
            alpha,
            beta,
        );
        match is_maximizing_player {
            true => {
                val = max(val, child_val);
                alpha = max(alpha, val);
            }
            false => {
                val = min(val, child_val);
                beta = min(beta, val);
            }
        }
        if alpha >= beta {
            break;
        }
    }
    val
}
