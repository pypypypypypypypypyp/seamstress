use crate::prelude::*;
use crate::boiler_plate::InputEvent;

const GRAVITY: f64 = 0.0035;
const DRAG: Vec2<f64> = Vec2{ x: 15.0, y: 2.0, };
const FRICTION: f64 = 0.5;

#[derive(Debug)]
pub struct GameState {
	pub player: Player,
	pub platforms: Vec<Platform>,
	pub enemies: Vec<Enemy>,
}

impl GameState {
	pub fn new() -> Self {
		Self {
			player: Player::new(),
			platforms: Vec::new(),
			enemies: Vec::new(),
		}
	}
	
	pub fn tick(&mut self, delta: f64) {
		self.player.do_movement(&self.platforms, &mut self.enemies, delta);
		if self.player.hp < 0.0 {
			self.player = Player::new();
		}
		let player = &mut self.player;
		let platforms = &self.platforms;
		let enemies = &mut self.enemies;
		for i in 0..enemies.len() {
			let mut e = enemies.remove(i);
			e.do_movement(player, platforms, enemies);
			enemies.insert(i, e);
		}
	}
	
	pub fn input_event(&mut self, e: InputEvent) {
		use InputEvent::*;
		match e {
			KeyDown(k) => match k.as_str() {
				"d" | "ArrowRight" => self.player.movement[0] = true,
				"a" | "ArrowLeft" => self.player.movement[1] = true,
				"w" | "ArrowUp" => self.player.movement[2] = true,
				"x" => self.player.detonate_thread(),
				_ => {},
			}
			KeyUp(k) => match k.as_str() {
				"d" | "ArrowRight" => self.player.movement[0] = false,
				"a" | "ArrowLeft" => self.player.movement[1] = false,
				"w" | "ArrowUp" => self.player.movement[2] = false,
				_ => {},
			}
			_ => {},
		}
	}
	
	pub fn render(&self) -> Vec<Vertex> {
		let mut r = Vec::new(); let v = &mut r; 
		self.player.render(v);
		let player_hp_frac = self.player.hp / self.player.max_hp();
		self.platforms.iter().for_each(|x| x.render(v, player_hp_frac));
		self.enemies.iter().for_each(|x| x.render(v));
		r
	}
}

pub trait BoundingBox {
	fn pos(&self) -> Vec2<f64>;
	fn size(&self) -> Vec2<f64>;
	fn vel(&self) -> Vec2<f64>;
	
	fn centre_pos(&self) -> Vec2<f64> { self.pos() + self.size() / 2.0 }
	fn left_edge(&self) -> f64 { self.pos().x - self.size().x }
	fn right_edge(&self) -> f64 { self.pos().x + self.size().x }
	fn bottom_edge(&self) -> f64 { self.pos().y + self.size().y }
	fn top_edge(&self) -> f64 { self.pos().y - self.size().y }
	
	fn collision_point_x(&self, other_vel: Vec2<f64>) -> Vec2<f64> {
		vec2(self.centre_pos().x + self.size().x / 2.0 * (self.vel().x - other_vel.x).signum(),self.centre_pos().y)
	}
	
	fn collision_point_y(&self, other_vel: Vec2<f64>) -> Vec2<f64> {
		vec2(self.centre_pos().x,self.centre_pos().y + self.size().y / 2.0 * (self.vel().y - other_vel.y).signum())
	}
	
	fn all_collision_points(&self) -> [Vec2<f64>; 4] {
		[
			vec2(self.centre_pos().x + self.size().x / 2.0,self.centre_pos().y),
			vec2(self.centre_pos().x - self.size().x / 2.0,self.centre_pos().y),
			vec2(self.centre_pos().x,self.centre_pos().y + self.size().y / 2.0),
			vec2(self.centre_pos().x,self.centre_pos().y - self.size().y / 2.0),
		]
	}
	
	fn point_collides(&self, point: Vec2<f64>, velocity: Vec2<f64>) -> Option<Vec2<f64>> { //returns how much it collides by, if any
		let p = point;
		let v = velocity - self.vel();
		let a = self.pos(); let b = self.pos() + self.size();
		if a.x < p.x && b.x > p.x && a.y < p.y && b.y > p.y {
			Some(vec2(
				if v.x.is_sign_negative() { b.x - p.x } else { p.x - a.x },
				if v.y.is_sign_negative() { b.y - p.y } else { p.y - a.y },
			))
		} else {
			None
		}
	}
}

pub trait Collider<T> {
	fn collides(&self, other: &T) -> bool;
}

impl<A: BoundingBox, B: BoundingBox> Collider<A> for B {
	fn collides(&self, other: &A) -> bool {
		let right = self.left_edge() > other.right_edge();
		let left = self.right_edge() < other.left_edge();
		let up = self.bottom_edge() < other.top_edge();
		let down = self.top_edge() > other.bottom_edge();
		!(right || left || up || down)
	}
}

/*#[derive(Debug,Default,Copy,Clone)]
pub struct AABB {
	pub pos: Vec2<f64>,
	pub size: Vec2<f64>,
	pub vel: Vec2<f64>,
}

impl BoundingBox for AABB {
	fn pos(&self) -> Vec2<f64> { self.pos }
	fn size(&self) -> Vec2<f64> { self.size }
	fn vel(&self) -> Vec2<f64> { self.vel }
}*/

#[derive(Debug,Default)]
pub struct Player {
	pub centre_pos: Vec2<f64>,
	pub vel: Vec2<f64>,
	pub movement: [bool; 4],
	pub grounded: f64,
	pub hp: f64,
	pub thread: Vec<Thread>,
	pub bob: Vec2<f64>, //to make thread wave up and down/left and right
	pub bob_dir: Vec2<i8>,
}

impl BoundingBox for Player {
	fn pos(&self) -> Vec2<f64> { self.centre_pos - self.size() / 2.0 }
	fn size(&self) -> Vec2<f64> { vec2(0.1, 0.2) }
	fn vel(&self) -> Vec2<f64> { self.vel }
}

impl Player {
	pub fn new() -> Self {
		let mut r = Self::default();
		r.bob_dir = vec2(1,1);
		r.hp = r.max_hp();
		r
	}
	
	pub fn move_speed(&self) -> f64 { if time() - self.grounded < 1.1 { 0.005 } else { 0.0025 } }
	pub fn jump_vel(&self) -> f64 { 0.13 }
	pub fn max_hp(&self) -> f64 { 30.0 }
	pub fn grounded_limit(&self) -> f64 { 5.0 } //time since touching a platform that we're still allowed to jump
	pub fn bob_rate(&self) -> Vec2<f64> { vec2(0.05,0.025) }
	pub fn bob_max(&self) -> Vec2<f64> { vec2(0.025,0.025) }
	
	pub fn detonate_thread(&mut self) {
		if !self.thread[self.thread.len()-1].detonated.is_some() {
			self.thread.iter_mut().enumerate().for_each(|(i, x)| x.detonate(i));
		}
	}
	
	pub fn render(&self, v: &mut Vec<Vertex>) {
		quad(v, self.pos(), 100, self.size(), [[1.0,1.0,1.0,1.0]; 4], [[0.0; 4]; 4], [[0.0; 3]; 4], 0.0, Default::default());
		for i in 0..self.thread.len().saturating_sub(1) {
			self.thread[i].render(v);
		}
	}
	
	pub fn do_movement(&mut self, platforms: &Vec<Platform>, enemies: &mut Vec<Enemy>, delta: f64) {
		let old_pos = self.centre_pos;
		let mut damaged = false;
		let movement_x = self.movement[0] as u8 as f64 - self.movement[1] as u8 as f64;
		self.vel.x += movement_x * self.move_speed();
		self.centre_pos.x += self.vel.x;
		for (is_enemy, object) in enemies.iter().map(|x| (true, x as &dyn BoundingBox)).chain(platforms.iter().map(|x| (false, x as &dyn BoundingBox))) { //collide enemies first so player doesn't get scronched into the floor
			let p = self.collision_point_x(object.vel());
			if let Some(correction) = object.point_collides(p, self.vel()) {
				if is_enemy { damaged = true; }
				let diff = (self.vel() - object.vel()).x;
				self.centre_pos.x -= diff.signum() * correction.x;
				self.vel.y -= self.vel.y.signum() * (diff.abs() * FRICTION).min(self.vel.y.abs());
				self.vel.x = object.vel().x;
			}
		}
		self.vel.y -= GRAVITY;
		if time() - self.grounded < self.grounded_limit() && self.movement[2] {
			self.vel.y += self.jump_vel();
			self.grounded = -self.grounded_limit();
		}
		self.centre_pos.y += self.vel.y;
		for (is_enemy, object) in enemies.iter().map(|x| (true, x as &dyn BoundingBox)).chain(platforms.iter().map(|x| (false, x as &dyn BoundingBox))) {
			let p = self.collision_point_y(object.vel());
			if let Some(correction) = object.point_collides(p, self.vel()) {
				if (self.vel - object.vel()).y.is_sign_negative() {
					self.grounded = time();
				} else if is_enemy { //only do damage when enemy on head, not when on head of enemy
					damaged = true;
				}
				let diff = (self.vel() - object.vel()).y;
				self.centre_pos.y -= diff.signum() * correction.y;
				self.vel.x -= self.vel.x.signum() * (diff.abs() * FRICTION).min(self.vel.x.abs());
				self.vel.y = object.vel().y;
			}
		}
		//TODO: this doesn't work
		//if still stuck in something, move upwards
		/*for object in enemies.iter().map(|x| x as &BoundingBox).chain(platforms.iter().map(|x| x as &BoundingBox)) {
			let p = self.all_collision_points();
			if p.iter().any(|&p| object.point_collides(p, self.vel()).is_some()) {
				self.centre_pos.y = object.top_edge() + self.size().y / 2.0;
			}
		}*/
		//let v = self.vel.magnitude();
		//self.vel *= 1.0 / (v * DRAG + 1.0);
		self.vel /= self.vel.abs() * DRAG + 1.0; //do drag on both axes seperately because... it feels better? idk
		self.hp += 0.05;
		if damaged {
			self.hp -= 1.0;
		}
		if self.thread.is_empty() {
			self.thread.push(Thread::new(old_pos, self.centre_pos + self.bob, time(), time()-1.0));
		}
		if self.vel.magnitude() > 0.001 {
			self.thread.insert(0, Thread::new(self.thread[0].to, self.centre_pos + self.bob, time(), self.thread[0].time));
		}
		let mut enemies_hit = HashSet::new();
		while !self.thread.is_empty() && (self.thread[self.thread.len() - 1].time + 500.0 < time() || self.thread[self.thread.len() - 1].detonated.map(|x| x < time()).unwrap_or(false)) {
			let thread = self.thread.pop().unwrap();
			if thread.detonated.is_some() {
				for i in 0..enemies.len() {
					if !enemies_hit.contains(&i) && thread.collides(&enemies[i]) {
						enemies_hit.insert(i);
					}
				}
			}
		}
		if self.thread.is_empty() {
			self.thread.push(Thread::new(old_pos, self.centre_pos + self.bob, time(), time()-1.0));
		}
		let mut enemies_hit = enemies_hit.iter().cloned().collect::<Vec<_>>();
		enemies_hit.sort();
		for &i in enemies_hit.iter().rev() {
			enemies[i].hp -= binomial(3) * 10.0;
			if enemies[i].hp <= 0.0 {
				enemies.remove(i);
			}
		}
		for i in 1..(self.thread.len()-1) {
			let parent = self.thread[i+1];
			let child = self.thread[i-1];
			self.thread[i].do_movement(delta, parent, child, self.centre_pos);
		}
		self.bob += self.bob_dir.f64()
			* vec2(binomial(2), binomial(2))
			* self.bob_rate()
			* self.vel.magnitude().sqrt()
			* ((self.bob_max().max(self.bob.abs()) * 1.2 - self.bob.abs()) / self.bob_max());
		self.bob_dir *= vec2(self.bob.x * self.bob_dir.f64().x > self.bob_max().x, self.bob.y * self.bob_dir.f64().y > self.bob_max().y).i8() * (-2) + 1;
		if self.vel.magnitude().sqrt() < 0.0001 && self.bob.y > -self.bob_max().y * 1.5 {
			self.bob.y -= self.thread[self.thread.len()-1].fall_rate(delta, self.centre_pos);
		}
		let n = 10.0;
		uniforms().set_cam_pos((uniforms().cam_pos * n + self.centre_pos) / (n+1.0));
	}
}

#[derive(Debug,Default)]
pub struct Platform {
	pub centre_pos: Vec2<f64>,
	pub size: Vec2<f64>,
}

impl BoundingBox for Platform {
	fn pos(&self) -> Vec2<f64> { self.centre_pos - self.size() / 2.0 }
	fn size(&self) -> Vec2<f64> { self.size }
	fn vel(&self) -> Vec2<f64> { Vec2::zero() }
}

impl Platform {
	pub fn render(&self, v: &mut Vec<Vertex>, player_hp_frac: f64) {
		quad(v, self.pos(), 0, self.size(), [[(1.0 - player_hp_frac as f32) / 2.0,0.0,0.0,1.0]; 4], [[0.0; 4]; 4], [[0.0; 3]; 4], 0.0, Default::default());
	}
}

#[derive(Debug,Default)]
pub struct Enemy {
	pub centre_pos: Vec2<f64>,
	pub vel: Vec2<f64>,
	pub grounded: f64,
	pub start_time: f64,
	pub hp: f64,
}

impl BoundingBox for Enemy {
	fn pos(&self) -> Vec2<f64> { self.centre_pos - self.size() / 2.0 }
	fn size(&self) -> Vec2<f64> { vec2(0.08,0.15) }
	fn vel(&self) -> Vec2<f64> { self.vel }
}

impl Enemy {
	pub fn new() -> Self {
		Self {
			start_time: time(),
			hp: Self::max_hp(),
			.. Default::default()
		}
	}
	
	fn max_hp() -> f64 {
		25.0
	}
	
	pub fn move_speed(&self) -> f64 { if time() - self.grounded < 1.1 { 0.0025 } else { 0.00125 } }
	
	pub fn do_movement(&mut self, player: &mut Player, platforms: &Vec<Platform>, enemies: &Vec<Enemy>) {
		let movement_x = if player.centre_pos.x > self.centre_pos.x { 1.0 } else { -1.0 };
		self.vel.x += movement_x * self.move_speed();
		self.centre_pos.x += self.vel.x;
		for object in enemies.iter().map(|x| x as &dyn BoundingBox).chain(platforms.iter().map(|x| x as &dyn BoundingBox)) {
			let p = self.collision_point_x(object.vel());
			if let Some(correction) = object.point_collides(p, self.vel()) {
				let diff = (self.vel() - object.vel()).x;
				self.centre_pos.x -= diff.signum() * correction.x;
				self.vel.y -= self.vel.y.signum() * (diff.abs() * FRICTION).min(self.vel.y.abs());
				self.vel.x = object.vel().x;
			}
		}
		self.vel.y -= GRAVITY;
		self.centre_pos.y += self.vel.y;
		for object in enemies.iter().map(|x| x as &dyn BoundingBox).chain(platforms.iter().map(|x| x as &dyn BoundingBox)) {
			let p = self.collision_point_y(object.vel());
			if let Some(correction) = object.point_collides(p, self.vel()) {
				if (self.vel - object.vel()).y.is_sign_negative() {
					self.grounded = time();
				}
				let diff = (self.vel() - object.vel()).y;
				self.centre_pos.y -= diff.signum() * correction.y;
				self.vel.x -= self.vel.x.signum() * (diff.abs() * FRICTION).min(self.vel.x.abs());
				self.vel.y = object.vel().y;
			}
		}
		self.vel /= self.vel.abs() * DRAG + 1.0;
	}
	
	pub fn render(&self, v: &mut Vec<Vertex>) {
		let t = self.start_time;
		let shine_rate = 0.03 * (7.0 + 3.0 * RandGen::new((self.start_time * 100.0) as u64).skip(100).f64()) / 10.0;
		let r = 0.2 / shine_rate;
		let start_time = [t,t+r,t+r*2.0,t+r*3.0];
		let mut x = [[0.0; 3]; 4];
		for i in 0..4 {
			for j in 0..3 {
				x[i][j] = start_time[i];
			}
		}
		let start_time = x;
		let mut color = [0.3,0.1,0.3,1.0];
		color.iter_mut().enumerate().for_each(|(i, x)| if i != 3 { *x *= (self.hp/Self::max_hp()) as f32; });
		quad(v, self.pos(), 1, self.size(), [color; 4], [[1.0,0.3,1.0,0.5]; 4], start_time, shine_rate, Default::default());
	}
}

#[derive(Debug,Default,Copy,Clone)]
pub struct Thread {
	pub from: Vec2<f64>,
	pub to: Vec2<f64>,
	pub time: f64,
	pub delta: f64,
	pub detonated: Option<f64>,
}

impl<T: BoundingBox> Collider<T> for Thread {
	fn collides(&self, other: &T) -> bool {
		other.point_collides(self.from, Vec2::zero()).is_some() || other.point_collides(self.to, Vec2::zero()).is_some()
	}
}

impl Thread {
	pub fn new(from: Vec2<f64>, to: Vec2<f64>, start_time: f64, last_time: f64) -> Self {
		Self {
			from,
			to,
			time: start_time,
			delta: start_time - last_time,
			detonated: None,
		}
	}
	
	pub fn width(&self) -> f64 { 0.01 }
	pub fn extra_padding(&self) -> f64 { 0.001 }
	pub fn fall_rate(&self, delta: f64, player_pos: Vec2<f64>) -> f64 {
		0.0001 * delta * (1.0 + (self.from.y - player_pos.y).max(0.0) * 2.0)
	}
	pub fn stretch_rate(&self) -> f64 { 0.00 }
	
	pub fn detonate(&mut self, i: usize) {
		self.detonated = Some(time() + 120.0 - i as f64 / 6.0);
	}
	
	pub fn render(&self, v: &mut Vec<Vertex>) {
		let dir = (self.to - self.from).normalize();
		let l = vec2(-dir.y,dir.x);
		let pos = [self.from + l * self.width(),self.from - l * self.width(),self.to + l * self.width(),self.to - l * self.width()];
		let start_time = [vec3(0f64,0.25,0.5),vec3(0.05,0.3,0.55),vec3(0f64,0.25,0.55),vec3(0.05,0.3,0.55)];
		let mut r = [Vertex::default(); 4];
		for i in 0..4 {
			let d = self.detonated.is_some();
			let shine_color = if d { [1.0,0.7,0.7,1.0] } else { [0.6,0.7,1.0,0.7] };
			let shine_bias = if d { [0.3,0.3,0.3] } else { [0.1,0.1,0.1] };
			let shine_rate = if d { 0.1 } else { 0.025 };
			r[i] = Vertex {
				pos: (pos[i] + dir * if i >= 2 { self.extra_padding() } else { -self.extra_padding() }).extend(get_z(10) + self.time.log10() * 0.0001).f32(),
				color: [0.7,0.7,0.7,1.0],
				shine_color,
				shine_bias,
				start_time: (start_time[i] * 60.0 + if i >= 2 { self.delta } else { 0.0 } - self.time).f32().into_array(),
				shine_rate,
			}
		}
		v.extend_from_slice(&quadify(r));
	}
	
	pub fn do_movement(&mut self, delta: f64, parent: Self, child: Self, player_pos: Vec2<f64>) {
		let x = self.fall_rate(delta, player_pos);
		self.from.y -= x;
		self.to.y -= x;
		self.from += (parent.from - self.from) * self.stretch_rate() * delta;
		self.to += (child.to - self.to) * self.stretch_rate() * delta;
	}
}
