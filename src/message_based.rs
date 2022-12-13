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
type ChannelData = (usize, Option<Size>);
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

/// This macro has two definitions that test the readiness of a component,
/// i.e., checking if a component in the machine is able to be pinged and/or
/// if a component that is a container for material has enough material to
/// complete the task.<br>
/// These two definitions include:<br>
/// 1. A check on a component that implements the Ping trait. This invocation
/// requires the following:<br>
/// 1a. An expression that evaluates to a usize representing a timeout for the
/// machine component to respond in milliseconds.<br>
/// 1b. A type that represents the machine component that implements the Ping
/// interface and it's associated functions.<br>
/// 2. A check on a component that implements the Ping and Capacity trait. This
/// invocation requires the following:<br>
/// 2a. An expression that evaluates to a usize representing a timeout for the
/// machine component to respond in milliseconds.<br>
/// 2b. An expression that evaluates to a Size enum representing the size of a
/// customer's cup.<br>
/// 2c. A type that represents the machine component that implements the Ping
/// interface and it's associated functions.
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

/// This macro contains one definition that creates a channel from the mpsc
/// library of the type ChannelData (defined at the top of the file).
/// This definition requires the following:
/// 1. An identifier that will be the name of the send channel being returned
/// from the channel constructor function.
/// 2. An identifier that will be the name of the receiver channel being
/// returned from the channel constructor function.
macro_rules! create_channel {
	($send_name: ident, $recv_name: ident) => {
		let ($send_name, $recv_name) = mpsc::channel::<ChannelData>();
	};
}
/// This macro contains two definitions for creating channel pipelines,
/// i.e. a function that runs on a thread that passes data through channels
/// to other functions running on another thread. These two pipelines are the
/// ending pipeline and the connector pipeline.<br>
/// 1. The end pipeline takes the following arguments:<br>
/// 1a. An identifier representing the name of the function.<br>
/// 1b. An identifier representing the name of the receiver channel that the 
/// pipe will be taking data from.<br>
/// 1c. A type representing a component struct that implements the ExecJob trait.<br>
/// 1d. An expression that represents a usize typed timeout<br>
/// 1e. An expression that represents a string that will be printed as a
/// success message. Can pass in a template to print out the cup ID.<br>
/// 2. the connector pipeline takes the following arguments (Mostly similar to
/// the end pipeline):<br>
/// 2a. An identifier representing the name of the function.<br>
/// 2b. An identifier representing the name of the receiver channel that the 
/// pipe will be taking data from.<br>
/// 2c. An identifier representing the name of the sender channel that the pipe
/// will be sending data to the next pipe.<br>
/// 2d. A type representing a component struct that implements the ExecJob trait.<br>
/// 2e. An expression that represents a usize typed timeout<br>
/// 2f. An expression that represents a string that will be printed as a
/// success message. Can pass in a template to print out the cup ID.<br>
macro_rules! create_pipeline {
	($func_name: ident, $recv_name: ident, $component: ty, $timeout: expr, $success_msg: expr) => {
		fn $func_name($recv_name: R<ChannelData>, worker: waitgroup::Worker) {
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
		fn $func_name($recv_name: R<ChannelData>, $send_name: S<ChannelData>, worker: waitgroup::Worker) {
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

fn start_coffee_maker(hopper_send: &S<ChannelData>, milk_send: &S<ChannelData>, timeout: usize, client_id: usize, size: Size) {
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
	// create a workgroup that will generate workers that will be passed to
	// threads. These workers will essentially block the exit of the main
	// thread until the workers passed to the threads are deallowcated.
	let wg = WaitGroup::new();
	let grind_beans_worker = wg.worker();
	let dispense_water_worker = wg.worker();
	let press_espresso_worker = wg.worker();
	let heat_milk_worker = wg.worker();
	let froth_milk_worker = wg.worker();
	// create a set of channels that will be passing data from thread to thread
	create_channel!(grind_send, grind_recv);
	create_channel!(water_send, water_recv);
	create_channel!(press_send, press_recv);
	create_channel!(milk_send, milk_recv);
	create_channel!(froth_send, froth_recv);
	// create a vector of cups that will be filled with coffee.
	let cups = ["Josh", "Sharon", "Moobly", "Tosh", "Mary"]
		.map(|name| { return Cup::new(Size::Medium, name.to_string()); });
	// create a vector of JoinHandles that will be executed using join
	// and return if the thread was created successfully.
	let threads = vec![
		thread::spawn(move || grind_coffee(grind_recv, water_send, grind_beans_worker)),
		thread::spawn(move || dispense_water(water_recv, press_send, dispense_water_worker)),
		thread::spawn(move || press_espresso(press_recv, press_espresso_worker)),
		thread::spawn(move || heat_milk(milk_recv, froth_send, heat_milk_worker)),
		thread::spawn(move || froth_milk(froth_recv, froth_milk_worker)),
	];
	// Attempt to start each of the threads. If there's an error starting a
	// thread, print out the error and return early.
	for t in threads {
		if let Err(e) = t.join() {
			if let Some(e) = e.downcast_ref::<&'static str>() {
				println!("Error starting thread: {}", e);
			} else {
				println!("Unknown Error starting thread: {:?}", e);
			}
			return;
		}
	}

	// for each cup of coffee weneed to make, run the checks on the machine
	// components necessary to know if the cup of coffee can be made or not.
	// if the checks pass, start making the coffee. If not, print error.
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