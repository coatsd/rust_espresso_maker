use std::{time, thread};
use std::sync::mpsc;
use std::fmt;
use std::boxed::Box;
use std::string::String;
use std::ops;
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
		fn $func_name($recv_name: R<u8>, timeout: u64) {
			match $recv_name.recv() {
				Ok(num) => println!($success_msg, num),
				Err(e) => println!("{}", e),
			}
		}
	};
	($func_name: ident, $recv_name: ident, $send_name: ident, $success_msg: expr) => {
		fn $func_name($recv_name: R<u8>, $send_name: S<u8>, timeout: u64) {
			match $recv_name.recv() {
				Ok(num) => match $send_name.send(num) {
					Ok(()) => println!($success_msg, num),
					Err(e) => println!("{}", e),
				},
				Err(e) => println!("{}", e),
			}
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

create_run_func!(grind_coffee, hopper_recv, water_send, "Coffee Ground for Client {}!");
create_run_func!(dispense_water, water_recv, press_send, "Water Dispensed for Client {}!");
create_run_func!(press_espresso, press_recv, "Espresso Pressed for Client {}!");

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
		let timeout = 101;
		create_channel!(grind_send, grind_recv);
		create_channel!(water_send, water_recv);
		create_channel!(press_send, press_recv);
		create_channel!(milk_send, milk_recv);
		create_channel!(froth_send, froth_recv);
		thread::spawn(move || grind_coffee(grind_recv, water_send, timeout));
		thread::spawn(move || dispense_water(water_recv, press_send, timeout));
	}
}