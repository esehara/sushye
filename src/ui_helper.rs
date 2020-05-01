use super::{
    gamelog, point_to_left, point_to_top, CombatStats, InBackpack, Map, Name, Player, Point,
    Position, RenderMode, Renderable, RunState, State, Viewshed, WantsToUseItem, TILESIZE,
    WINDOWSIZE_HEIGHT,
};
use ggez::graphics;
use ggez::graphics::*;
use ggez::mint::Point2;
use ggez::Context;
use specs::prelude::*;

pub fn p_to_map(p: f32) -> i32 {
    (p as i32) / TILESIZE
}
pub fn map_to_p(p: i32) -> f32 {
    (p * TILESIZE) as f32
}

pub fn draw_text(
    ctx: &mut Context,
    textfs: &Vec<graphics::TextFragment>,
    pos: Point2<f32>,
    font: graphics::Font,
) {
    let mut text = graphics::Text::default();
    for textf in textfs {
        text.add(textf.clone().font(font));
    }

    graphics::queue_text(ctx, &text, Point2 { x: 0.0, y: 0.0 }, None);
    graphics::draw_queued_text(
        ctx,
        graphics::DrawParam::default().dest(pos),
        None,
        graphics::FilterMode::Linear,
    )
    .expect("Cannot draw Text");
}

pub fn draw_tile_text(
    ctx: &mut Context,
    textf: graphics::TextFragment,
    x: i32,
    y: i32,
    font: graphics::Font,
) {
    let mut text = graphics::Text::default();
    text.add(textf.clone().font(font).scale(Scale {
        x: TILESIZE as f32,
        y: TILESIZE as f32,
    }));

    graphics::queue_text(ctx, &text, Point2 { x: 0.0, y: 0.0 }, None);
    graphics::draw_queued_text(
        ctx,
        graphics::DrawParam::default().dest(Point2 {
            x: map_to_p(x),
            y: map_to_p(y),
        }),
        None,
        graphics::FilterMode::Linear,
    )
    .expect("Cannot draw Text");
}

pub fn draw_message_window(ctx: &mut Context, ecs: &World, font: Font) {
    let log = ecs.fetch::<gamelog::GameLog>();

    let windowpoint_x = map_to_p(1);
    let window_height = map_to_p(2);
    let rect = Rect::new(
        windowpoint_x,
        fix_p_to_bottom(window_height),
        map_to_p(38),
        window_height,
    );

    let r1 = Mesh::new_rectangle(
        ctx,
        DrawMode::fill(),
        rect,
        Color::new(0.05, 0.05, 0.05, 1.0),
    )
    .unwrap();
    graphics::draw(ctx, &r1, graphics::DrawParam::default()).expect("Cannot draw Messege Window");

    let r2 = Mesh::new_rectangle(
        ctx,
        DrawMode::stroke(1.0),
        rect,
        Color::new(1.0, 1.0, 1.0, 0.0),
    )
    .unwrap();
    graphics::draw(ctx, &r2, graphics::DrawParam::default()).expect("Cannot draw Messege Window");

    let max_message_size = window_height / 16.0;
    for (i, s) in log.entries.iter().rev().enumerate() {
        let mut fix_s: Vec<TextFragment> = Vec::new();
        for t in s {
            let fix_i = (i as f32) * 0.3;
            match t.color {
                None => fix_s.push(t.clone().color(Color::new(1.0, 1.0, 1.0, 1.0 - fix_i))),
                Some(c) => {
                    let newt = TextFragment::new(t.text.clone()).color(Color::new(
                        c.r,
                        c.g,
                        c.b,
                        c.a - fix_i,
                    ));
                    fix_s.push(newt);
                }
            }
        }

        if (i as f32) < max_message_size {
            draw_text(
                ctx,
                &fix_s,
                Point2 {
                    x: windowpoint_x,
                    y: fix_p_to_bottom(window_height) + ((i as f32) * 16.0),
                },
                font,
            )
        }
    }
}

pub fn draw_inventory_window(
    state: &State,
    ctx: &mut Context,
    ecs: &World,
    font: Font,
    mouse_pos: Point2<f32>,
    render_mode: &RenderMode,
) {
    let player_entity = ecs.fetch::<Entity>();
    let names = ecs.read_storage::<Name>();
    let backpack = ecs.read_storage::<InBackpack>();
    let renderables = ecs.read_storage::<Renderable>();

    let inventory_len = (&backpack, &names, &renderables)
        .join()
        .filter(|item| item.0.owner == *player_entity)
        .count();
    let start_window_x = map_to_p(1);
    let start_window_y = map_to_p(1);
    let item_image_x = 4.0 * 16.0;
    let rect = Rect::new(
        start_window_x,
        start_window_y,
        map_to_p(28),
        map_to_p(2) + map_to_p(inventory_len as i32),
    );

    let r1 =
        Mesh::new_rectangle(ctx, DrawMode::fill(), rect, Color::new(0.1, 0.1, 0.1, 1.0)).unwrap();

    let r2 = Mesh::new_rectangle(
        ctx,
        DrawMode::stroke(2.0),
        rect,
        Color::new(1.0, 1.0, 1.0, 1.0),
    )
    .unwrap();

    graphics::draw(ctx, &r1, graphics::DrawParam::default())
        .expect("Cannot draw Inventory window -> Fill");
    graphics::draw(ctx, &r2, graphics::DrawParam::default())
        .expect("Cannot draw inventory window -> Stroke");

    draw_text(
        ctx,
        &vec![TextFragment::new("所持品(inventory)")],
        Point2 {
            x: start_window_x + map_to_p(1),
            y: start_window_y + 16.0,
        },
        font,
    );

    let mut j = 0;
    for (_pack, name, use_image) in (&backpack, &names, &renderables)
        .join()
        .filter(|item| item.0.owner == *player_entity)
    {
        let line_y = start_window_y + map_to_p(2) + map_to_p(j);

        draw_text(
            ctx,
            &vec![TextFragment::new(format!("({})", ((97 + j) as u8) as char))],
            Point2 {
                x: start_window_x + map_to_p(1),
                y: line_y,
            },
            font,
        );
        match render_mode {
            RenderMode::Tile => {
                graphics::draw(
                    ctx,
                    state.images.get(&use_image.image).unwrap(),
                    graphics::DrawParam::default().dest(Point2 {
                        x: start_window_x + item_image_x,
                        y: line_y - 8.0,
                    }),
                )
                .expect("Cannot draw Item image.");
            }
            RenderMode::Unicode => {
                let mut text = graphics::Text::default();
                text.add(
                    state
                        .enum_to_unicode(&use_image)
                        .clone()
                        .font(font)
                        .scale(Scale {
                            x: TILESIZE as f32,
                            y: TILESIZE as f32,
                        }),
                );

                graphics::queue_text(ctx, &text, Point2 { x: 0.0, y: 0.0 }, None);
                graphics::draw_queued_text(
                    ctx,
                    graphics::DrawParam::default().dest(Point2 {
                        x: start_window_x + item_image_x,
                        y: line_y - 8.0,
                    }),
                    None,
                    graphics::FilterMode::Linear,
                )
                .expect("Cannot draw Text");
            }
        }

        draw_text(
            ctx,
            &vec![TextFragment::new(format!(" - {}", &name.name))],
            Point2 {
                x: start_window_x + map_to_p(3),
                y: line_y,
            },
            font,
        );
        j += 1;
    }
}

pub fn draw_mouse_pos(ctx: &mut Context, x: f32, y: f32) {
    let r = &Mesh::new_rectangle(
        ctx,
        DrawMode::stroke(2.0),
        Rect::new(
            map_to_p(p_to_map(x)) as f32,
            map_to_p(p_to_map(y)) as f32,
            TILESIZE as f32,
            TILESIZE as f32,
        ),
        Color::new(1.0, 1.0, 0.0, 1.0),
    )
    .unwrap();

    graphics::draw(ctx, r, graphics::DrawParam::default()).unwrap();
}

pub fn draw_object_focus_rect(ctx: &mut Context, ecs: &World, x: f32, y: f32, pos: &Position) {
    let player_pos = ecs.fetch::<Point>();
    let fix_player_pos_left = point_to_left(*player_pos) + 1;
    let fix_player_pos_top = point_to_top(*player_pos) + 1;
    if p_to_map(x) + fix_player_pos_left == pos.x && p_to_map(y) + fix_player_pos_top == pos.y {
        let r = Mesh::new_rectangle(
            ctx,
            DrawMode::fill(),
            Rect::new(
                map_to_p(p_to_map(x)),
                map_to_p(p_to_map(y)),
                TILESIZE as f32,
                TILESIZE as f32,
            ),
            Color::new(1.0, 0.0, 1.0, 0.2),
        )
        .unwrap();
        graphics::draw(ctx, &r, graphics::DrawParam::default()).unwrap();
    }
}

pub fn draw_aoe_radius(ctx: &mut Context, ecs: &World, x: f32, y: f32, radius: i32) {
    let map = ecs.fetch::<Map>();
    let player_pos = ecs.fetch::<Point>();
    let fix_player_pos_left = point_to_left(*player_pos) + 1;
    let fix_player_pos_top = point_to_top(*player_pos) + 1;
    let blast_tiles = bracket_lib::prelude::field_of_view(
        Point {
            x: p_to_map(x) + fix_player_pos_left,
            y: p_to_map(y) + fix_player_pos_top,
        },
        radius,
        &*map,
    );
    for tile_idx in blast_tiles.iter() {
        let r = Mesh::new_rectangle(
            ctx,
            DrawMode::fill(),
            Rect::new(
                map_to_p(tile_idx.x - fix_player_pos_left),
                map_to_p(tile_idx.y - fix_player_pos_top),
                TILESIZE as f32,
                TILESIZE as f32,
            ),
            Color::new(1.0, 0.0, 0.0, 0.2),
        )
        .unwrap();
        graphics::draw(ctx, &r, graphics::DrawParam::default()).unwrap();
    }
}

pub fn draw_ranged_target(ecs: &World, ctx: &mut Context, range: i32) {
    let available_cells = inside_range(ecs, range);
    let player_pos = ecs.fetch::<Point>();
    let fix_player_pos_left = point_to_left(*player_pos) + 1;
    let fix_player_pos_top = point_to_top(*player_pos) + 1;

    for idx in available_cells.iter() {
        let distance = bracket_lib::prelude::DistanceAlg::Pythagoras.distance2d(*player_pos, *idx);
        let r: Mesh;
        if distance <= range as f32 {
            r = Mesh::new_rectangle(
                ctx,
                DrawMode::fill(),
                Rect::new(
                    map_to_p(idx.x - fix_player_pos_left),
                    map_to_p(idx.y - fix_player_pos_top),
                    TILESIZE as f32,
                    TILESIZE as f32,
                ),
                Color::new(1.0, 1.0, 0.0, 0.1),
            )
            .unwrap();
        } else {
            r = Mesh::new_rectangle(
                ctx,
                DrawMode::fill(),
                Rect::new(
                    map_to_p(idx.x - fix_player_pos_left),
                    map_to_p(idx.y - fix_player_pos_top),
                    TILESIZE as f32,
                    TILESIZE as f32,
                ),
                Color::new(1.0, 0.0, 0.0, 0.1),
            )
            .unwrap();
        }
        graphics::draw(ctx, &r, graphics::DrawParam::default()).unwrap();
    }
}

pub fn draw_tooltip_with_mouse_motion(
    ctx: &mut Context,
    ecs: &World,
    x: f32,
    y: f32,
    font: graphics::Font,
) {
    let map = ecs.fetch::<Map>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();
    let mouse_pos_x = p_to_map(x);
    let mouse_pos_y = p_to_map(y);

    let player_pos = ecs.fetch::<Point>();
    let fix_player_pos_left = point_to_left(*player_pos) + 1;
    let fix_player_pos_top = point_to_top(*player_pos) + 1;

    let mut tooltip: Vec<String> = Vec::new();

    if mouse_pos_x >= map.width || mouse_pos_y >= map.height {
        return;
    }

    for (name, position) in (&names, &positions).join() {
        if position.x == mouse_pos_x + fix_player_pos_left
            && position.y == mouse_pos_y + fix_player_pos_top
        {
            tooltip.push(name.name.to_string());
        }
    }

    if !tooltip.is_empty() {
        for (i, s) in tooltip.iter().enumerate() {
            let rect = Rect::new(
                map_to_p(mouse_pos_x + 1),
                map_to_p(mouse_pos_y) + ((i as f32) * 16.0),
                ((s.len() + 3) as f32) * 8.0,
                16.0,
            );

            let r2 =
                Mesh::new_rectangle(ctx, DrawMode::fill(), rect, Color::new(0.2, 0.2, 0.2, 1.0))
                    .unwrap();
            graphics::draw(ctx, &r2, graphics::DrawParam::default())
                .expect("Cannot draw Messege Window");
            let tooltip_text = vec![graphics::TextFragment::new(format!("<- {}", s))];
            draw_text(
                ctx,
                &tooltip_text,
                Point2 {
                    x: map_to_p(mouse_pos_x + 1),
                    y: map_to_p(mouse_pos_y) + ((i as f32) * 16.0),
                },
                font,
            );
        }
    }
}

pub fn fix_p_to_bottom(ph: f32) -> f32 {
    map_to_p(WINDOWSIZE_HEIGHT) - (ph + 16.0)
}

fn inside_range(ecs: &World, range: i32) -> Vec<Point> {
    let player_entity = ecs.fetch::<Entity>();
    let player_pos = ecs.fetch::<Point>();

    let viewsheds = ecs.read_storage::<Viewshed>();
    let visible = viewsheds.get(*player_entity);

    let mut available_cells: Vec<Point> = Vec::new();

    if let Some(visible) = visible {
        for idx in visible.visible_tiles.iter() {
            let distance =
                bracket_lib::prelude::DistanceAlg::Pythagoras.distance2d(*player_pos, *idx);
            if distance <= range as f32 {
                available_cells.push(*idx);
            }
        }
    }
    available_cells
}

pub fn try_target_object(ecs: &World, x: f32, y: f32, item: Entity, range: i32) -> RunState {
    let player_pos = ecs.fetch::<Point>();
    let mouse_pos_x = p_to_map(x);
    let mouse_pos_y = p_to_map(y);
    let fix_player_pos_left = point_to_left(*player_pos) + 1;
    let fix_player_pos_top = point_to_top(*player_pos) + 1;

    let available_cells = inside_range(ecs, range);
    for idx in available_cells.iter() {
        if idx.x == mouse_pos_x + fix_player_pos_left && idx.y == mouse_pos_y + fix_player_pos_top {
            let mut intent = ecs.write_storage::<WantsToUseItem>();
            intent
                .insert(
                    *ecs.fetch::<Entity>(),
                    WantsToUseItem {
                        item,
                        target: Some(Point::new(
                            mouse_pos_x + fix_player_pos_left,
                            mouse_pos_y + fix_player_pos_top,
                        )),
                    },
                )
                .unwrap();
            return RunState::PlayerTurn;
        }
    }
    RunState::AwaitingInput
}
