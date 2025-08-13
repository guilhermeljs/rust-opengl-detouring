
use std::{cell::RefCell, sync::{Arc, Mutex, RwLock}};

use egui::pipeline::{draw_egui, initialize_egui_glow, render_egui};
use graphics::opengl::pipeline::{GenericPipeline, MutablePipeline};
use memory_sys::{dll_entrypoint, internal::{library::get_dll_proc, logging::debug_log}};
use windows::Win32::Graphics::Gdi::HDC;

use ::egui::Context;
use ::egui::Window;
use ::egui::pos2;

mod graphics;
mod opengl;
mod egui;
mod lua;

fn initialize_opengl() {
    let pointer = get_dll_proc("OPENGL32.DLL", "wglSwapBuffers");

    let mut render_pipeline: GenericPipeline<HDC> = GenericPipeline::empty();

    let screen = Screen::<Options>::new(Options::default(), |options,ctx| {
        Window::new("Test").current_pos(pos2(100.0, 500.0)).show(ctx, |ui| {
            ui.label("Version: 0.1.0");
            ui.checkbox(&mut options.test, "Hello World");
        });
    });
    screen.add_to_pipeline(&mut render_pipeline);



    let mut init_pipeline: GenericPipeline<HDC> = GenericPipeline::empty();
    init_pipeline.add_stage(|hdc| {
        initialize_egui_glow(hdc);
    });



    let _ = graphics::opengl::pipeline::configure_init_pipeline(init_pipeline);
    let _ = graphics::opengl::pipeline::configure_render_pipeline(render_pipeline);

    if let Ok(pointer) = pointer {
        debug_log(format!("Pointer found: {:02X}" , pointer.0).as_str());
        graphics::opengl::hook::hook_opengl();
    }
}
struct Screen<T>
where
    T: Sync + Send + 'static,
{
    options_context: Arc<Mutex<T>>,
    render: Arc<Mutex<Box<dyn FnMut(&mut T, &Context) + Sync + Send + 'static>>>,
}

impl<T> Screen<T>
where
    T: Sync + Send + 'static,
{
    pub fn new<F>(options: T, render: F) -> Self
    where
        F: FnMut(&mut T, &Context) + Sync + Send + 'static,
    {
        Screen {
            options_context: Arc::new(Mutex::new(options)),
            render: Arc::new(Mutex::new(Box::new(render))),
        }
    }

    pub fn add_to_pipeline(&self, pipeline: &mut GenericPipeline<HDC>) {
        let render_clone = self.render.clone();
        let options_clone = self.options_context.clone();

        pipeline.add_stage(move |_hdc| {
            let render = render_clone.clone();
            let options_context = options_clone.clone();

            let egui = draw_egui(move |ctx| {
                let mut options = options_context.lock().unwrap();
                let mut render_fn = render.lock().unwrap();

                (&mut *render_fn)(&mut *options, ctx);
            });

            if let Ok(egui) = egui {
                let _ = render_egui(egui);
            }
        });
    }
}

#[derive(Default)]
struct Options {
    test: bool,
}

dll_entrypoint!();
fn main() {
    initialize_opengl();
}
