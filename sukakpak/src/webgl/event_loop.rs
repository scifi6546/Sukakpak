use super::{ControlFlow, EventLoopTrait, WindowEvent};

use js_sys::Function;
use log::{info, Level};
use nalgebra::Vector2;
use wasm_bindgen::{prelude::*, JsValue};
pub struct EventLoop {}
impl EventLoopTrait for EventLoop {
    fn new(_: Vector2<u32>) -> Self {
        console_log::init_with_level(Level::Debug);
        Self {}
    }
    fn run<F: 'static + FnMut(WindowEvent, &mut ControlFlow)>(self, mut game_fn: F) {
        let mut flow = ControlFlow::Continue;
        // safe befause wasm does not run in parallel
        unsafe {
            game_closure = Some(Closure::wrap(Box::new(loop_fn)));
            game_fn_data = Some(Box::new(game_fn));
        }
        loop_fn();
    }
}
/// points to function to run game.
static mut game_fn_data: Option<Box<dyn FnMut(WindowEvent, &mut ControlFlow)>> = None;
/// points to `loop_fn`
static mut game_closure: Option<Closure<dyn FnMut()>> = None;

#[wasm_bindgen]
pub fn loop_fn() {
    let game_fn = unsafe { game_fn_data.as_mut().unwrap() };
    let mut flow = ControlFlow::Continue;
    game_fn(WindowEvent::RunGameLogic, &mut flow);
    if flow == ControlFlow::Quit {
        panic!()
    }
    let game_closure_value: &JsValue = unsafe { game_closure.as_ref() }.unwrap().as_ref();
    web_sys::window()
        .expect("failed to get window")
        .request_animation_frame(
            Function::try_from(game_closure_value)
                .expect("failed to convert event loop function to javascript object"),
        )
        .expect("failed to request frame");
}
