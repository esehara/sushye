use super::{
    try_drop_item, try_use_item, CombatStats, InBackpack, MainMenuState, Name, Player, Renderable,
    RunState, TILESIZE, WINDOWSIZE_HEIGHT, WINDOWSIZE_WIDTH,
};

use ggez::graphics;
use ggez::Context;

use gfx_core::{handle::RenderTargetView, memory::Typed};
use gfx_device_gl;

use imgui::*;
use imgui_gfx_renderer::*;

use specs::prelude::*;
use std::path::Path;
use std::time::Instant;

#[derive(Copy, Clone, PartialEq, Debug, Default)]
struct MouseState {
    pos: (i32, i32),
    pressed: (bool, bool, bool),
    wheel: f32,
}

pub struct ImGuiWrapper {
    pub imgui: imgui::Context,
    pub has_save: bool,
    pub inventory_window_show: bool,
    pub renderer: Renderer<gfx_core::format::Rgba8, gfx_device_gl::Resources>,
    last_frame: Instant,
    mouse_state: MouseState,
    show_popup: bool,
    texture_id: Option<TextureId>,
}

impl ImGuiWrapper {
    pub fn new(ctx: &mut Context) -> Self {
        // Create the imgui object
        let mut imgui = imgui::Context::create();
        let (factory, gfx_device, _, _, _) = graphics::gfx_objects(ctx);
        let unicode_font = imgui.fonts().add_font(&[FontSource::TtfData {
            data: include_bytes!("../resources/unifont-13.ttf"),
            size_pixels: 16.0,
            config: Some(FontConfig {
                rasterizer_multiply: 1.75,
                glyph_ranges: FontGlyphRanges::japanese(),
                ..FontConfig::default()
            }),
        }]);

        // Shaders
        let shaders = {
            let version = gfx_device.get_info().shading_language;
            if version.is_embedded {
                if version.major >= 3 {
                    Shaders::GlSlEs300
                } else {
                    Shaders::GlSlEs100
                }
            } else if version.major >= 4 {
                Shaders::GlSl400
            } else if version.major >= 3 {
                Shaders::GlSl130
            } else {
                Shaders::GlSl110
            }
        };

        // Renderer
        let renderer = Renderer::init(&mut imgui, &mut *factory, shaders).unwrap();
        let has_save = Path::new("./savegame.json").exists();
        // Create instace
        Self {
            imgui,
            has_save,
            inventory_window_show: false,
            renderer,
            last_frame: Instant::now(),
            mouse_state: MouseState::default(),
            show_popup: false,
            texture_id: None,
        }
    }

    fn initialize_for_draw(&mut self, ctx: &mut Context, hidpi_factor: f32) {
        self.update_mouse();
        let now = Instant::now();
        let delta = now - self.last_frame;
        let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
        self.last_frame = now;

        let (draw_width, draw_height) = graphics::drawable_size(ctx);
        self.imgui.io_mut().display_size = [draw_width, draw_height];
        self.imgui.io_mut().display_framebuffer_scale = [hidpi_factor, hidpi_factor];
        self.imgui.io_mut().delta_time = delta_s;
    }

    const INVENTORY_WINDOW_WIDTH_SIZE: f32 = (TILESIZE * 24) as f32;
    const INVENTORY_WINDOW_HEIGHT_SIZE: f32 = (TILESIZE * 8) as f32;
    const STATES_WINDOW_WIDTH_SIZE: f32 = (TILESIZE * 8) as f32;
    const STATES_WINDOW_HEIGHT_SIZE: f32 = (TILESIZE * WINDOWSIZE_HEIGHT) as f32;

    pub fn render(&mut self, ctx: &mut Context, ecs: &mut World, hidpi_factor: f32) {
        self.initialize_for_draw(ctx, hidpi_factor);
        let ui = self.imgui.frame();
        let has_save;
        {
            has_save = self.has_save.clone();
        }
        let mut newrunstate;
        {
            newrunstate = *ecs.fetch_mut::<RunState>();
        }
        let mut not_title = true;
        let mut push_any_inventory_button: Option<i32> = None;
        {
            match newrunstate {
                RunState::MainMenu { state: _ } => {
                    // -------------------------------------
                    // main window
                    // -------------------------------------
                    Window::new(im_str!("Start Menu"))
                        .flags(
                            WindowFlags::NO_TITLE_BAR
                                | WindowFlags::NO_RESIZE
                                | WindowFlags::NO_MOVE,
                        )
                        .size([300.0, 300.0], imgui::Condition::Always)
                        .position([100.0, 100.0], imgui::Condition::Always)
                        .build(&ui, || {
                            ui.text(im_str!("Sushy -- Typical Roguelike!!"));
                            ui.text(im_str!("ようこそ、Sushyeの世界へ！"));
                            ui.separator();
                            if ui.small_button(im_str!("Start")) {
                                newrunstate = RunState::MainMenu {
                                    state: MainMenuState::NewGame,
                                };
                            }
                            if has_save {
                                if ui.small_button(im_str!("Load Game")) {
                                    if Path::new("./savegame.json").exists() {
                                        newrunstate = RunState::MainMenu {
                                            state: MainMenuState::LoadGame,
                                        };
                                    }
                                }
                            }

                            if ui.small_button(im_str!("Quit")) {
                                newrunstate = RunState::MainMenu {
                                    state: MainMenuState::Quit,
                                };
                            }
                        });
                    not_title = false;
                }
                _ => {}
            }
            if not_title {
                if self.inventory_window_show {
                    // -----------------------------
                    // Inventory Window
                    // -----------------------------
                    let player_entity = ecs.fetch::<Entity>();
                    let names = ecs.read_storage::<Name>();
                    let backpack = ecs.read_storage::<InBackpack>();
                    let mut j = 0;
                    Window::new(im_str!("Inventory"))
                        .size(
                            [
                                ImGuiWrapper::INVENTORY_WINDOW_WIDTH_SIZE,
                                ImGuiWrapper::INVENTORY_WINDOW_HEIGHT_SIZE,
                            ],
                            imgui::Condition::Always,
                        )
                        .position([16.0, 16.0], imgui::Condition::Always)
                        .opened(&mut self.inventory_window_show)
                        .flags(
                            WindowFlags::NO_COLLAPSE
                                | WindowFlags::NO_RESIZE
                                | WindowFlags::NO_MOVE,
                        )
                        .build(&ui, || {
                            for (_pack, name) in (&backpack, &names)
                                .join()
                                .filter(|item| item.0.owner == *player_entity)
                            {
                                let key_char = ((97 + j) as u8) as char;
                                let button_text = im_str!("{}", key_char);
                                if ui.small_button(&button_text) {
                                    push_any_inventory_button = Some(j);
                                }
                                ui.same_line(0.0);
                                ui.text(format!("- {}", name.name.to_string()));
                                j += 1;
                            }
                        });
                }
                // ---------------------------------------
                // Player States Window
                // ---------------------------------------
                let combat_stats = ecs.read_storage::<CombatStats>();
                let players = ecs.read_storage::<Player>();
                for (_player, stats) in (&players, &combat_stats).join() {
                    // Window
                    Window::new(im_str!("Player"))
                        .flags(WindowFlags::NO_COLLAPSE)
                        .size(
                            [
                                ImGuiWrapper::STATES_WINDOW_WIDTH_SIZE - 32.0,
                                ImGuiWrapper::STATES_WINDOW_HEIGHT_SIZE - (32.0 * 4.0),
                            ],
                            imgui::Condition::FirstUseEver,
                        )
                        .position(
                            [
                                ((TILESIZE * WINDOWSIZE_WIDTH) as f32)
                                    - (ImGuiWrapper::STATES_WINDOW_WIDTH_SIZE + 64.0),
                                16.0,
                            ],
                            imgui::Condition::FirstUseEver,
                        )
                        .build(&ui, || {
                            ui.text(format!("HP: {} / {}", stats.hp, stats.max_hp));
                            ProgressBar::new((stats.hp as f32) / (stats.max_hp as f32)).build(&ui);
                            ui.spacing();
                            if CollapsingHeader::new(&ui, im_str!("Equipment"))
                                .open_on_arrow(true)
                                .default_open(true)
                                .build()
                            {
                                ui.text("Weapon:");
                                ui.same_line(0.0);
                                ui.text_colored([0.0, 1.0, 1.0, 1.0], "None");

                                ui.text("Shield:");
                                ui.same_line(0.0);
                                ui.text_colored([0.0, 1.0, 1.0, 1.0], "None");
                            };
                        });
                }
            }
        }
        match push_any_inventory_button {
            None => {}
            Some(j) => match newrunstate {
                RunState::ShowInventory => {
                    newrunstate = try_use_item(ecs, j, &mut self.inventory_window_show);
                }
                RunState::ShowDropItem => {
                    newrunstate = try_drop_item(ecs, j, &mut self.inventory_window_show);
                }
                _ => {}
            },
        }
        if !self.inventory_window_show
            && (newrunstate == RunState::ShowDropItem || newrunstate == RunState::ShowInventory)
        {
            newrunstate = RunState::AwaitingInput;
        }

        {
            let mut runstate = ecs.fetch_mut::<RunState>();
            *runstate = newrunstate;
        }

        //

        // Render
        let (factory, _, encoder, _, render_target) = graphics::gfx_objects(ctx);
        let draw_data = ui.render();
        self.renderer
            .render(
                &mut *factory,
                encoder,
                &mut RenderTargetView::new(render_target.clone()),
                draw_data,
            )
            .unwrap();
    }

    fn update_mouse(&mut self) {
        self.imgui.io_mut().mouse_pos =
            [self.mouse_state.pos.0 as f32, self.mouse_state.pos.1 as f32];

        self.imgui.io_mut().mouse_down = [
            self.mouse_state.pressed.0,
            self.mouse_state.pressed.1,
            self.mouse_state.pressed.2,
            false,
            false,
        ];

        self.imgui.io_mut().mouse_wheel = self.mouse_state.wheel;
        self.mouse_state.wheel = 0.0;
    }

    pub fn update_mouse_pos(&mut self, x: f32, y: f32) {
        self.mouse_state.pos = (x as i32, y as i32);
    }

    pub fn update_mouse_down(&mut self, pressed: (bool, bool, bool)) {
        self.mouse_state.pressed = pressed;
    }
}
