#![allow(non_snake_case)]

use std::cell::RefCell;
use std::rc::Rc;
use egui::{ClippedMesh, CtxRef};
use egui_wgpu_backend::{BackendError, ScreenDescriptor};
use egui_wgpu_backend::RenderPass;
use pixels::PixelsContext;
use winit::window::Window;
use crate::gui_commands::GuiCommands;

pub struct Gui {
    context: CtxRef,
    state: egui_winit::State,
    descriptor: ScreenDescriptor,
    renderPass: RenderPass,
    meshes: Vec<ClippedMesh>,
    components: GuiComponents
}

impl Gui {
    pub fn new(width: u32, height: u32, scale: f32, commands: Rc<RefCell<GuiCommands>>, pixels: &pixels::Pixels) -> Self {
        Gui {
            context: CtxRef::default(),
            state: egui_winit::State::from_pixels_per_point(scale),
            descriptor: ScreenDescriptor {
                physical_width: width,
                physical_height: height,
                scale_factor: scale
            },
            renderPass: RenderPass::new(pixels.device(), pixels.render_texture_format(), 1),
            meshes: Vec::new(),
            components: GuiComponents::new(commands)
        }
    }

    pub fn prepareGui(&mut self, window: &Window) -> () {
        let input = self.state.take_egui_input(window);
        let (output, commands) = self.context.run(input, |context| {
            self.components.buildUi(context);
        });

        self.state.handle_output(window, &self.context, output);
        self.meshes = self.context.tessellate(commands);
    }

    pub fn render(&mut self, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView, context: &PixelsContext) -> Result<(), BackendError> {
        self.renderPass.update_texture(&context.device, &context.queue, &self.context.font_image());
        self.renderPass.update_user_textures(&context.device, &context.queue);
        self.renderPass.update_buffers(&context.device, &context.queue, &self.meshes, &self.descriptor);
        return self.renderPass.execute(encoder, target, &self.meshes,&self.descriptor, None);
    }

    pub fn handleEvent(&mut self, event: &winit::event::WindowEvent) -> () {
        self.state.on_event(&self.context, event);
    }


}

struct GuiComponents {
    aboutVisible: bool,
    commands: Rc<RefCell<GuiCommands>>
}

impl GuiComponents {
    pub fn new(commands: Rc<RefCell<GuiCommands>>) -> Self {
        GuiComponents {
            aboutVisible: false,
            commands
        }
    }


    pub fn buildUi(&mut self, context: &CtxRef) -> () {
        egui::TopBottomPanel::top("menubar").show(context, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("About...").clicked() {
                        self.aboutVisible = true;
                        ui.close_menu();
                    }

                    if ui.button("Open").clicked() {
                        *self.commands.borrow_mut() = GuiCommands::LoadGame;
                        ui.close_menu();
                    }

                    ui.separator();

                    if ui.button("Save State").clicked() {
                        *self.commands.borrow_mut() = GuiCommands::SaveState
                    }

                    if ui.button("Load State").clicked() {
                        *self.commands.borrow_mut() = GuiCommands::LoadState
                    }

                    ui.separator();

                    if ui.button("Quit").clicked() {
                        std::process::exit(0);
                    }
                });
            })
        });



        egui::Window::new("Welcome to RustyNES!")
            .open(&mut self.aboutVisible)
            .show(context, |ui| {
                ui.label("This is a hobby NES emulator written in Rust.");
                ui.label("Thanks, and have fun!")
            });
    }
}
