use std::{time, thread};
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

struct Cup {
	// size is used to check if there are enough ingredients for order.
	size: Size,
	contents: Box<Vec<Ingredient>>,
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
		}
	}
}
impl Cup {
	fn new(s: Size) -> Self {
		Cup {
			size: s,
			contents: Box::new(Vec::<Ingredient>::new()),	
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
		let ($send_name, $recv_name) = mpsc::channel::<u8>();
	};
}

macro_rules! create_run_func {
	($func_name: ident, $recv_name: ident, $success_msg: expr) => {
		fn $func_name($recv_name: R<u8>, worker: waitgroup::Worker) {
			while let Ok(num) = $recv_name.recv() {
				println!($success_msg, num)
			}
			drop(worker);
		}
	};
	($func_name: ident, $recv_name: ident, $send_name: ident, $success_msg: expr) => {
		fn $func_name($recv_name: R<u8>, $send_name: S<u8>, worker: waitgroup::Worker) {
			while let Ok(num) = $recv_name.recv() {
				match $send_name.send(num) {
					Ok(()) => println!($success_msg, num),
					Err(e) => println!("{}", e),
				}
			}
			drop(worker);
		}
	};
}

fn run_checks(t_o: u64, s: Size) -> [Result<(), String>; 5] {
	[
		check_machine!(t_o, s, CoffeeHopper),
		check_machine!(t_o, s, WaterTank),
		check_machine!(t_o, EspressoPress),
		check_machine!(t_o, s, MilkTank),
		check_machine!(t_o, Frother)
	]
}

fn start_coffee_maker(hopper_send: &S<u8>, milk_send: &S<u8>, timeout: u64, customer_id: u8) {
	if (timeout < 50) {
		println!("Client {} Start Coffee Timeout!", customer_id);
	}
	match hopper_send.send(customer_id) {
		Ok(()) => println!("Client {} Coffee Beans Started!", customer_id),
		Err(e) => println!("Error Starting Client {} Coffee Beans!\n{}", customer_id, e),
	}
	match milk_send.send(customer_id) {
		Ok(()) => println!("Client {} Milk Started!", customer_id),
		Err(e) => println!("Error starting Client {} Milk!\n{}", customer_id, e),
	}
}

create_run_func!(grind_coffee, hopper_recv, water_send, "Coffee Ground for Client {}!");
create_run_func!(dispense_water, water_recv, press_send, "Water Dispensed for Client {}!");
create_run_func!(press_espresso, press_recv, "Espresso Pressed for Client {}!");
create_run_func!(heat_milk, milk_recv, froth_send, "Milk heated for Client {}!");
create_run_func!(froth_milk, froth_recv, "Milk frothed for Client {}!");

async fn do_five_times() {
	let timeout = 101;
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
	thread::spawn(move || grind_coffee(grind_recv, water_send, grind_beans_worker));
	thread::spawn(move || dispense_water(water_recv, press_send, dispense_water_worker));
	thread::spawn(move || press_espresso(press_recv, press_espresso_worker));
	thread::spawn(move || heat_milk(milk_recv, froth_send, heat_milk_worker));
	thread::spawn(move || froth_milk(froth_recv, froth_milk_worker));
	for id in 1..5 {
		start_coffee_maker(&grind_send, &milk_send, timeout, id);
	}
	drop(grind_send);
	drop(milk_send);
	wg.wait().await;
}

pub fn message_based_main() {
	let c = Cup::new(Size::Medium);
	let mut passed_checks = true;
	for check in run_checks(100, c.size).iter() {
		match check {
			Err(e) => { passed_checks = false; println!("{}", e); },
			_ => println!("Passed!"),
		}
	}

	if passed_checks {
		// do stuff
		block_on(do_five_times());
	}
}