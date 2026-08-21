use std::iter::empty;
use itertools::Either;

use crate::draw::element::{
    AdicEl,
    PathDInstruction,
    CircleEl, PathEl, TextEl,
};
use super::ClockShape;


pub (super) fn clock_face_instructions(clock_shape: &ClockShape) -> impl Iterator<Item=AdicEl> {

    let (cx, cy) = clock_shape.center();
    let face_radius = clock_shape.radius();

    let mut face_views = vec![
        AdicEl::Circle(CircleEl{
            class: Some("clock-circle".to_string()),
            cx, cy, r: face_radius,
        })
    ];

    let zidx = clock_shape.zero_valuation_idx();
    for (idx, r) in clock_shape.hand_positions().iter().map(|pos| pos.radius).enumerate() {
        let class = if clock_shape.show_zero_val_circle() && zidx.is_some_and(|i| idx == i) {
            Some("clock-zero-val-circle".to_string())
        } else if clock_shape.show_val_circles() {
            Some("clock-val-circle".to_string())
        } else {
            None
        };
        if let Some(class) = class {
            face_views.push(AdicEl::Circle(CircleEl{
                class: Some(class),
                cx, cy, r,
            }));
        }
    }

    face_views.into_iter()

}

pub (super) fn clock_hand_instructions(clock_shape: &ClockShape) -> impl Iterator<Item=AdicEl> {

    // let path = svg_el::Path::new()
    //     .set("fill", "none")
    //     .set("stroke", "black")
    //     .set("stroke-width", 3)
    //     .set("d", data);

    let (cx, cy) = clock_shape.center();
    clock_shape.hand_positions().into_iter().flat_map(move |hand_pos| {
        let target = hand_pos.head_position;
        let hand_data = vec![
            PathDInstruction::Move((cx, cy)),
            PathDInstruction::Line(target),
        ];
        [
            AdicEl::Path(PathEl{
                class: Some("clock-hand-path".to_string()),
                d: hand_data,
            }),
            AdicEl::Circle(CircleEl{
                class: Some("clock-head-circle".to_string()),
                cx: target.0, cy: target.1, r: 0.5,
            })
        ]
    })

}

pub (super) fn clock_mark_instructions(clock_shape: &ClockShape) -> impl Iterator<Item=AdicEl> {

    let (cx, cy) = clock_shape.center();
    if clock_shape.tick_positions().len() <= MAX_TICK_MARKS {
        Either::Left(clock_shape.tick_positions().into_iter().map(move |tick_pos| {
            let source = (cx + (tick_pos.0 - cx) * 0.95, cy + (tick_pos.1 - cy) * 0.95);
            let target = tick_pos;
            let mark_instructions = vec![
                PathDInstruction::Move(source),
                PathDInstruction::Line(target),
            ];
            AdicEl::Path(PathEl{
                class: Some("clock-hand-path".to_string()),
                d: mark_instructions,
            })
        }))
    } else {
        Either::Right(empty())
    }

}

pub (super) fn clock_label_instructions(clock_shape: &ClockShape) -> impl Iterator<Item=AdicEl> {

    if clock_shape.show_tick_labels() && clock_shape.tick_positions().len() <= MAX_TICK_LABELS {

        let label_font_size = 4.;
        // Font is in pt and non-font is not, so possibly include a magic number multiplier
        let magic_font_multiplier = 1.;
        let label_size = label_font_size * magic_font_multiplier;
        let label_style = format!("position: fixed; font-size: {label_size}pt;");

        let (cx, cy) = clock_shape.center();
        Either::Left(clock_shape.tick_positions().into_iter().enumerate().map(move |(tick, tick_pos)| {
            let adjusted = (
                (tick_pos.0 - cx) * 0.1,
                (tick_pos.1 - cy) * 0.1
            );
            // let adjusted = (0., 0.);
            AdicEl::Text(TextEl{
                content: tick.to_string(),
                class: Some("tick-label".to_string()),
                style: Some(label_style.clone()),
                x: tick_pos.0, y: tick_pos.1,
                dx: adjusted.0, dy: adjusted.1,
            })
        }))

    } else {
        Either::Right(empty())
    }

}


const MAX_TICK_MARKS: usize = 100;
const MAX_TICK_LABELS: usize = 25;
