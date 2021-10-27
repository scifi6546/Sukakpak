use super::{ControlFlow, EventLoopTrait, WindowEvent};

use js_sys::Function;
use log::{info, Level};
use nalgebra::Vector2;
use wasm_bindgen::{prelude::*, JsValue};
pub struct EventLoop {}
impl EventLoopTrait for EventLoop {
    fn new(_: Vector2<u32>) -> Self {
        console_log::init_with_level(Level::Debug).expect("failed to initilize console log");
        Self {}
    }
    fn run<F: 'static + FnMut(WindowEvent, &mut ControlFlow)>(self, game_fn: F) {
        // safe befause wasm does not run in parallel
        unsafe {
            GAME_CLOSURE = Some(Closure::wrap(Box::new(loop_fn)));
            GAME_FN = Some(Box::new(game_fn));
        }
        loop_fn();
    }
}
/// points to function to run game.
static mut GAME_FN: Option<Box<dyn FnMut(WindowEvent, &mut ControlFlow)>> = None;
/// points to `loop_fn`
static mut GAME_CLOSURE: Option<Closure<dyn FnMut()>> = None;

#[wasm_bindgen]
pub fn loop_fn() {
    let game_fn = unsafe { GAME_FN.as_mut().unwrap() };
    let mut flow = ControlFlow::Continue;
    game_fn(WindowEvent::RunGameLogic, &mut flow);
    if flow == ControlFlow::Quit {
        panic!()
    }
    let game_closure_value: &JsValue = unsafe { GAME_CLOSURE.as_ref() }.unwrap().as_ref();
    web_sys::window()
        .expect("failed to get window")
        .request_animation_frame(
            Function::try_from(game_closure_value)
                .expect("failed to convert event loop function to javascript object"),
        )
        .expect("failed to request frame");
}
