use bevy::{app::AppExit, prelude::*};
use bevy_egui::{
    egui::{self, epaint::Shadow, Align2},
    EguiContexts, EguiPlugin,
};

use crate::{game::GameState, GlobalState, Settings};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, States)]
pub enum MenuState {
    /// Main Menu
    #[default]
    Main,
}

pub struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<MenuState>()
            .add_plugins(EguiPlugin)
            .add_systems(
                Update,
                (
                    display_main_menu
                        .run_if(in_state(GlobalState::Menu).and_then(in_state(MenuState::Main))),
                    display_game_over
                        .run_if(in_state(GlobalState::Game).and_then(in_state(GameState::Ended))),
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
    mut next_global_state: ResMut<NextState<GlobalState>>,
    mut exit_events: EventWriter<AppExit>,
) {
    let ctx = contexts.ctx_mut();
    global_settings(ctx);
    create_menu_window("Sweeper 3D").show(ctx, |ui| {
        ui.allocate_ui(egui::Vec2::new(0.0, 0.0), |ui| {
            ui.vertical_centered(|ui| {
                ui.horizontal_centered(|ui| {
                    if ui.add(egui::Button::new("Small")).clicked() {
                        settings.field_size = [3, 3, 3];
                        next_global_state.set(GlobalState::Game);
                    }
                    if ui.add(egui::Button::new("Medium")).clicked() {
                        settings.field_size = [5, 5, 5];
                        next_global_state.set(GlobalState::Game);
                    }
                    if ui.add(egui::Button::new("Large")).clicked() {
                        settings.field_size = [10, 10, 10];
                        next_global_state.set(GlobalState::Game);
                    }
                    if ui.add(egui::Button::new("Custom")).clicked() {
                        settings.field_size = [3, 3, 3];
                        next_global_state.set(GlobalState::Game);
                    }
                });
                ui.add(egui::Button::new("Settings"));
                if ui.add(egui::Button::new("Quit")).clicked() {
                    exit_events.send(AppExit);
                }
            });
        });
    });
}

fn display_game_over(
    mut contexts: EguiContexts,
    mut next_global_state: ResMut<NextState<GlobalState>>,
    mut next_menu_state: ResMut<NextState<MenuState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut exit_events: EventWriter<AppExit>,
) {
    let ctx = contexts.ctx_mut();
    global_settings(ctx);
    create_menu_window("Game Over").show(ctx, |ui| {
        ui.allocate_ui(egui::Vec2::new(0.0, 0.0), |ui| {
            ui.vertical_centered(|ui| {
                ui.horizontal_centered(|ui| {
                    if ui.add(egui::Button::new("Restart")).clicked() {
                        next_game_state.set(GameState::Start);
                    }
                    if ui.add(egui::Button::new("Main Menu")).clicked() {
                        next_menu_state.set(MenuState::Main);
                        next_global_state.set(GlobalState::Menu);
                    }
                    if ui.add(egui::Button::new("Quit")).clicked() {
                        exit_events.send(AppExit);
                    }
                });
            });
        });
    });
}
