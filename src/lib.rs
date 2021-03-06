#![feature(decl_macro,const_float_bits_conv)]
mod prelude; use prelude::*;
mod boiler_plate;
mod vertex;
mod game_state;

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
	initial_setup();
	setup_input_events();
	let (gl, u_loc) = setup_rendering().expect(l!());
	
	let mut game_state = GameState::new();
	game_state.platforms.push(Platform {
		centre_pos: vec2(0.0,-0.8),
		size: vec2(200.0,0.4),
	});
	game_state.platforms.push(Platform {
		centre_pos: vec2(-1.0,0.0),
		size: vec2(0.3,1.0),
	});
	game_state.platforms.push(Platform {
		centre_pos: vec2(1.0,0.0),
		size: vec2(0.3,1.0),
	});
	game_state.platforms.push(Platform {
		centre_pos: vec2(-0.5,-0.2),
		size: vec2(0.4,0.1),
	});
	game_state.platforms.push(Platform {
		centre_pos: vec2(0.0,0.2),
		size: vec2(0.4,0.1),
	});
	game_state.platforms.push(Platform {
		centre_pos: vec2(0.5,0.6),
		size: vec2(0.4,0.1),
	});
	
	let f = None.rc(); let g = f.clone();
	*f.borrow_mut() = Some(Closure::wrap(Box::new(move|| {
		let delta = 1.0;
		uniforms().add_time(delta);
		//run_callbacks();
		if time() % 100.0 < 0.1 {
			if game_state.enemies.len() < 3 {
				game_state.enemies.push(Enemy {
					centre_pos: vec2(0.7,0.8),
					.. Enemy::new()
				})
			}
		}
		game_state.tick(delta);
		for event in input_events().drain(..) {
			game_state.input_event(event);
		}
		render(&gl, &u_loc, &game_state.render());
		request_animation_frame(g.borrow().as_ref().unwrap());
	}) as Box<dyn FnMut()>));
	request_animation_frame(f.borrow().as_ref().unwrap());
	Ok(())
}
