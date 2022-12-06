use std::thread;
use waitgroup::WaitGroup;
use std::sync::mpsc;
use std::fmt;
use std::boxed::Box;
use std::string::String;
use std::ops;
use futures::executor::block_on;
use crate::machine_components::*;

type S<T> = mpsc::Sender<T>;
type R<T> = mpsc::Receiver<T>;
const TIMEOUT: usize = 101;

struct Cup {
	// size is used to check if there are enough ingredients for order.
	size: Size,
	contents: Box<Vec<Ingredient>>,
	client: String,
}
impl fmt::Display for Cup {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		use Ingredient::*;
		let contents: String = self.contents.iter()
			.map(|i| match i {Espresso => " Espresso ", Milk => " Milk "})
			.fold(String::from(""), |acc, i| acc + i);
		write!(f, "Size: {}, Contents: {}", self.size, contents)
	}
}
impl ops::Add<Ingredient> for Cup {
	type Output = Self;

	// memory inefficient - creates new cup instead of altering current one.
	fn add(self, i: Ingredient) -> Self {
		let mut c = self.contents;
		c.push(i);
		Self {
			size: self.size,
			contents: c,
			client: self.client,
		}
	}
}
impl Cup {
	fn new(s: Size, c: String) -> Self {
		Cup {
			size: s,
			contents: Box::new(Vec::<Ingredient>::new()),
			client: c,
		}
	}
	fn add_ingredient(mut self, i: Ingredient) {
		self.contents.push(i);
	}
}

macro_rules! check_machine {
	($t_o:expr, $m:ty) => {
		if let Result::Err(e) = <$m>::ping($t_o) {
			Err(e)
		} else {
			Ok(())
		}
	};
	($t_o:expr, $s:expr, $m:ty) => {
		if let Result::Err(e) = <$m>::ping($t_o) {
			Err(e)
		} else {
			if let Result::Err(e) = <$m>::check_capacity($s) {
				Err(e)
			} else {
				Ok(())
			}
		}
	};
}

macro_rules! create_channel {
	($send_name: ident, $recv_name: ident) => {
		let ($send_name, $recv_name) = mpsc::channel::<(usize, Option<Size>)>();
	};
}

macro_rules! create_pipeline {
	($func_name: ident, $recv_name: ident, $component: ty, $timeout: expr, $success_msg: expr) => {
		fn $func_name($recv_name: R<(usize, Option<Size>)>, worker: waitgroup::Worker) {
			while let Ok((cup_id, size)) = $recv_name.recv() {
				match <$component>::exec_job($timeout, size) {
					Err(e) => println!("{}", e),
					_ => println!($success_msg, cup_id),
				}
			}
			drop(worker);
		}
	};
	($func_name: ident, $recv_name: ident, $send_name: ident, $component: ty, $timeout: expr, $success_msg: expr) => {
		fn $func_name($recv_name: R<(usize, Option<Size>)>, $send_name: S<(usize, Option<Size>)>, worker: waitgroup::Worker) {
			while let Ok((cup_id, size)) = $recv_name.recv() {
				match <$component>::exec_job($timeout, size) {
					Err(e) => println!("{}", e),
					_ => match $send_name.send((cup_id, size)) {
						Ok(()) => println!($success_msg, cup_id),
						Err(e) => println!("{}", e),
					}
				}
			}
			drop(worker);
		}
	};
}

fn run_checks(t_o: usize, s: Size) -> [Result<(), String>; 5] {
	[
		check_machine!(t_o, s, CoffeeHopper),
		check_machine!(t_o, s, WaterTank),
		check_machine!(t_o, EspressoPress),
		check_machine!(t_o, s, MilkTank),
		check_machine!(t_o, Frother)
	]
}

fn start_coffee_maker(hopper_send: &S<(usize, Option<Size>)>, milk_send: &S<(usize, Option<Size>)>, timeout: usize, client_id: usize, size: Size) {
	if timeout < 50 {
		println!("Client {} Start Coffee Timeout!", client_id);
	}
	match hopper_send.send((client_id, Option::Some(size))) {
		Ok(()) => println!("Client {} Coffee Beans Started!", client_id),
		Err(e) => println!("Error Starting Client {} Coffee Beans!\n{}", client_id, e),
	}
	match milk_send.send((client_id, Option::Some(size))) {
		Ok(()) => println!("Client {} Milk Started!", client_id),
		Err(e) => println!("Error starting Client {} Milk!\n{}", client_id, e),
	}
}

create_pipeline!(grind_coffee, hopper_recv, water_send, CoffeeHopper, TIMEOUT, "Coffee Ground for Client {}!");
create_pipeline!(dispense_water, water_recv, press_send, WaterTank, TIMEOUT, "Water Dispensed for Client {}!");
create_pipeline!(press_espresso, press_recv, EspressoPress, TIMEOUT, "Espresso Pressed for Client {}!");
create_pipeline!(heat_milk, milk_recv, froth_send, MilkTank, TIMEOUT, "Milk heated for Client {}!");
create_pipeline!(froth_milk, froth_recv, Frother, TIMEOUT, "Milk frothed for Client {}!");

async fn do_five_times() {
	let wg = WaitGroup::new();
	let grind_beans_worker = wg.worker();
	let dispense_water_worker = wg.worker();
	let press_espresso_worker = wg.worker();
	let heat_milk_worker = wg.worker();
	let froth_milk_worker = wg.worker();
	create_channel!(grind_send, grind_recv);
	create_channel!(water_send, water_recv);
	create_channel!(press_send, press_recv);
	create_channel!(milk_send, milk_recv);
	create_channel!(froth_send, froth_recv);
	let cups = ["Josh", "Sharon", "Moobly", "Tosh", "Mary"]
		.map(|name| { return Cup::new(Size::Medium, name.to_string()); });

	let threads = vec![
		thread::spawn(move || grind_coffee(grind_recv, water_send, grind_beans_worker)),
		thread::spawn(move || dispense_water(water_recv, press_send, dispense_water_worker)),
		thread::spawn(move || press_espresso(press_recv, press_espresso_worker)),
		thread::spawn(move || heat_milk(milk_recv, froth_send, heat_milk_worker)),
		thread::spawn(move || froth_milk(froth_recv, froth_milk_worker)),
	];

	for t in threads {
		if let Err(e) = t.join() {
			if let Some(e) = e.downcast_ref::<&'static str>() {
				println!("Error starting thread: {}", e);
			} else {
				println!("Unknown Error starting thread: {:?}", e);
			}
		}
	}

	for (id, cup) in cups.iter().enumerate() {
		let mut passed = true;
		for check in run_checks(TIMEOUT, cup.size) {
			match check {
				Err(e) => {
					passed = false;
					println!("{}", e);
				},
				_ => (),
			}
		}
		if passed {
			start_coffee_maker(&grind_send, &milk_send, TIMEOUT, id, cup.size);
		} else {
			println!("Cannot make {}'s Coffee!", cup.client);
		}
	}
	drop(grind_send);
	drop(milk_send);
	wg.wait().await;
}

pub fn message_based_main() {
	// do stuff
	block_on(do_five_times());
}