use bevy::{app::AppExit, prelude::*};
use bevy_egui::{
    egui::{self, epaint::Shadow, Align2},
    EguiContexts, EguiPlugin,
};

use crate::{settings::Safety, GameState, Settings};

pub struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin).add_systems(
            Update,
            (
                display_main_menu.run_if(in_state(GameState::MenuMain)),
                display_custom_menu.run_if(in_state(GameState::MenuCustom)),
                display_settings_menu.run_if(in_state(GameState::MenuSettings)),
                display_game_over.run_if(in_state(GameState::GameOver)),
            ),
        );
    }
}

fn global_settings(ctx: &mut egui::Context) {
    ctx.style_mut(|style| {
        style.text_styles.insert(
            egui::TextStyle::Heading,
            egui::FontId::new(32.0, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::new(24.0, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::new(24.0, egui::FontFamily::Proportional),
        );
        style.spacing.item_spacing = egui::Vec2::new(5.0, 5.0);
    });
    let mut visuals = egui::Visuals::dark();
    visuals.window_shadow = Shadow::NONE;
}

fn create_menu_window<'a>(title: impl Into<egui::WidgetText>) -> egui::Window<'a> {
    egui::Window::new(title)
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .collapsible(false)
        .movable(false)
        .resizable(false)
}

fn display_main_menu(
    mut contexts: EguiContexts,
    mut settings: ResMut<Settings>,
    mut next_state: ResMut<NextState<GameState>>,
    mut exit_events: EventWriter<AppExit>,
) {
    let ctx = contexts.ctx_mut();
    global_settings(ctx);
    create_menu_window("Sweeper 3D").show(ctx, |ui| {
        ui.allocate_ui(egui::Vec2::new(0.0, 0.0), |ui| {
            ui.vertical_centered(|ui| {
                ui.horizontal_centered(|ui| {
                    if ui.add(egui::Button::new("Small")).clicked() {
                        settings.set_if_neq(Settings::small());
                        next_state.set(GameState::GameStart);
                    }
                    if ui.add(egui::Button::new("Medium")).clicked() {
                        settings.set_if_neq(Settings::medium());
                        next_state.set(GameState::GameStart);
                    }
                    if ui.add(egui::Button::new("Large")).clicked() {
                        settings.set_if_neq(Settings::large());
                        next_state.set(GameState::GameStart);
                    }
                    if ui.add(egui::Button::new("Custom")).clicked() {
                        settings.set_if_neq(Settings::default());
                        next_state.set(GameState::MenuCustom);
                    }
                });
                if ui.add(egui::Button::new("Settings")).clicked() {
                    next_state.set(GameState::MenuSettings);
                }
                if ui.add(egui::Button::new("Quit")).clicked() {
                    exit_events.send(AppExit);
                }
            });
        });
    });
}

fn display_custom_menu(
    mut contexts: EguiContexts,
    mut settings: ResMut<Settings>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let (field_size, mine_density, _) = settings.fields_mut();
    let ctx = contexts.ctx_mut();
    global_settings(ctx);
    create_menu_window("Custom Game").show(ctx, |ui| {
        ui.allocate_ui(egui::Vec2::new(0.0, 0.0), |ui| {
            ui.vertical_centered(|ui| {
                ui.horizontal_centered(|ui| {
                    ui.add(egui::Label::new("Size:"));
                    ui.add(egui::DragValue::new(&mut field_size[0]).clamp_range(1..=20));
                    ui.add(egui::DragValue::new(&mut field_size[1]).clamp_range(1..=20));
                    ui.add(egui::DragValue::new(&mut field_size[2]).clamp_range(1..=20));
                });
                ui.horizontal_centered(|ui| {
                    ui.add(egui::Label::new("Mine Density:"));
                    ui.add(
                        egui::Slider::new(mine_density, 0.01..=1.0)
                            .min_decimals(2)
                            .max_decimals(2),
                    );
                });
                ui.horizontal_centered(|ui| {
                    if ui.add(egui::Button::new("Start")).clicked() {
                        next_state.set(GameState::GameStart);
                    }
                    if ui.add(egui::Button::new("Back")).clicked() {
                        next_state.set(GameState::MenuMain);
                    }
                });
            });
        });
    });
}

fn display_settings_menu(
    mut contexts: EguiContexts,
    mut settings: ResMut<Settings>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let (_, _, safety) = settings.fields_mut();
    let ctx = contexts.ctx_mut();
    global_settings(ctx);
    create_menu_window("Settings").show(ctx, |ui| {
        ui.allocate_ui(egui::Vec2::new(0.0, 0.0), |ui| {
            ui.vertical_centered(|ui| {
                ui.horizontal_centered(|ui| {
                    ui.label("First Block Safety:");
                    ui.radio_value(safety, Safety::Safe, "Guaranteed Safe")
                        .on_hover_text_at_pointer(concat!(
                            "The first block cleared is guaranteed to be safe, ",
                            "but may only reveal one space."
                        ));
                    ui.radio_value(safety, Safety::Clear, "Guaranteed Clear")
                        .on_hover_text_at_pointer(
                            "The first block cleared is guaranteed to reveal more than one space.",
                        );
                    ui.radio_value(safety, Safety::Random, "Random")
                        .on_hover_text_at_pointer(
                            "No safety guarantees - the first block cleared might contain a mine.",
                        );
                });
                ui.horizontal_centered(|ui| {
                    if ui.add(egui::Button::new("Back")).clicked() {
                        next_state.set(GameState::MenuMain);
                    }
                });
            });
        });
    });
}

fn display_game_over(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<GameState>>,
    mut exit_events: EventWriter<AppExit>,
) {
    let ctx = contexts.ctx_mut();
    global_settings(ctx);
    egui::Window::new("Game Over")
        .anchor(Align2::CENTER_BOTTOM, [0.0, 0.0])
        .collapsible(false)
        .movable(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.allocate_ui(egui::Vec2::new(0.0, 0.0), |ui| {
                ui.vertical_centered(|ui| {
                    ui.horizontal_centered(|ui| {
                        if ui.add(egui::Button::new("Restart")).clicked() {
                            next_state.set(GameState::GameStart);
                        }
                        if ui.add(egui::Button::new("Main Menu")).clicked() {
                            next_state.set(GameState::MenuMain);
                        }
                        if ui.add(egui::Button::new("Quit")).clicked() {
                            exit_events.send(AppExit);
                        }
                    });
                });
            });
        });
}
