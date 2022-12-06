use std::fmt;
use std::{time, thread};
use rand::{thread_rng, Rng};
use std::sync::mpsc;

// The "amount" of beans in coffee hopper in ounces.
const BEANAMOUNT: f32 = 2.0;
// The "amount" of water in tank in ounces.
const WATERAMOUNT: f32 = 2.0;
// The "amount" of milk in tank in ounces.
const MILKAMOUNT: f32 = 10.0;
// The "capacity" of each hopper or tank.
const CAPACITY: f32 = 64.0;
type S<T> = mpsc::Sender<T>;
type R<T> = mpsc::Receiver<T>;

pub enum Ingredient {
	Espresso,
	Milk,
}
impl fmt::Display for Ingredient {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		use Ingredient::*;
		match self {
			Espresso => write!(f, "Espresso"),
			Milk => write!(f, "Milk"),
		}
	}
}

#[derive(Copy, Clone)]
pub enum Size {
	Small,
	Medium,
	Large,
}
impl fmt::Display for Size {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		use Size::*;
		match self {
			Small => write!(f, "8 oz."),
			Medium => write!(f, "12 oz."),
			Large => write!(f, "16 oz."),
		}
	}
}

pub trait Ping {
	fn ping(timeout: usize) -> Result<(), String>;
}
pub trait Capacity {
	fn check_capacity(s: Size) -> Result<(), String>;
}

pub trait ExecJob {
	fn exec_job(timeout: usize, size: Option<Size>) -> Result<(), String>;
}

pub struct CoffeeHopper;
impl Ping for CoffeeHopper {
	fn ping(timeout: usize) -> Result<(), String> {
		let rng = thread_rng().gen_range(2..100);
		thread::sleep(time::Duration::from_millis(rng));
		if rng as usize > timeout {
			Err("CoffeeHopper Component Not Responding".to_string())
		} else {
			Ok(())
		}
	}
}
impl Capacity for CoffeeHopper {
	fn check_capacity(s: Size) -> Result<(), String> {
		use Size::*;
		let s: f32 = match s {
			Small => 1.0,
			Medium => 2.0,
			Large => 3.0,
		};
		if s <= BEANAMOUNT {
			Ok(())	
		} else {
			Err("Not enough coffee beans in CoffeeHopper".to_string())
		}
	}
}
impl CoffeeHopper {
	fn grind_beans(timeout: usize, size: Option<Size>) -> Result<(), String> {
		if let Err(e) = CoffeeHopper::ping(timeout) {
			return Err(e.to_string());
		}
		match size {
			Some(s) => {
				if let Err(e) = CoffeeHopper::check_capacity(s) {
					return Err(e.to_string());
				}
			},
			None => (),
		}
		Ok(())
	}
}
impl ExecJob for CoffeeHopper {
	fn exec_job(timeout: usize, size: Option<Size>) -> Result<(), String> {
		CoffeeHopper::grind_beans(timeout, size)
	}
}

pub struct WaterTank;
impl Ping for WaterTank {
	fn ping(timeout: usize) -> Result<(), String> {
		let rng = thread_rng().gen_range(2..100);
		thread::sleep(time::Duration::from_millis(rng));
		if rng as usize > timeout {
			Err("WaterTank Component Not Responding".to_string())
		} else {
			Ok(())
		}
	}
}
impl Capacity for WaterTank {
	fn check_capacity(s: Size) -> Result<(), String> {
		use Size::*;
		let s: f32 = match s {
			Small => 1.0,
			Medium => 2.0,
			Large => 3.0,
		};
		if s <= WATERAMOUNT {
			Ok(())	
		} else {
			Err("Not enough water in WaterTank".to_string())
		}
	}
}
impl WaterTank {
	fn dispense(timeout: usize, size: Option<Size>) -> Result<(), String> {
		if let Err(e) = WaterTank::ping(timeout) {
			return Err(e.to_string());
		}
		match size {
			Some(s) => {
				if let Err(e) = WaterTank::check_capacity(s) {
					return Err(e.to_string());
				}
			},
			None => (),
		}
		Ok(())
	}
}
impl ExecJob for WaterTank {
	fn exec_job(timeout: usize, size: Option<Size>) -> Result<(), String> {
		WaterTank::dispense(timeout, size)
	}
}

pub struct EspressoPress;
impl Ping for EspressoPress {
	fn ping(timeout: usize) -> Result<(), String> {
		let rng = thread_rng().gen_range(2..100);
		thread::sleep(time::Duration::from_millis(rng));
		if rng as usize > timeout {
			Err("EspressoPress Component Not Responding".to_string())
		} else {
			Ok(())
		}
	}
}
impl EspressoPress {
	fn press(timeout: usize) -> Result<(), String> {
		if let Err(e) = EspressoPress::ping(timeout) {
			return Err(e.to_string())
		}
		Ok(())
	}
}
impl ExecJob for EspressoPress {
	fn exec_job(timeout: usize, _: Option<Size>) -> Result<(), String> {
		EspressoPress::press(timeout)
	}
}

pub struct MilkTank;
impl Ping for MilkTank {
	fn ping(timeout: usize) -> Result<(), String> {
		let rng = thread_rng().gen_range(2..100);
		thread::sleep(time::Duration::from_millis(rng));
		if rng as usize > timeout {
			Err("MilkTank Component Not Responding".to_string())
		} else {
			Ok(())
		}
	}
}
impl Capacity for MilkTank {
	fn check_capacity(s: Size) -> Result<(), String> {
		use Size::*;
		let s: f32 = match s {
			Small => 7.0,
			Medium => 10.0,
			Large => 13.0,
		};
		if s <= MILKAMOUNT {
			Ok(())
		} else {
			Err("Not enough milk in MilkTank".to_string())
		}
	}
}
impl MilkTank {
	fn dispense(timeout: usize, size: Option<Size>) -> Result<(), String> {
		if let Err(e) = MilkTank::ping(timeout) {
			return Err(e.to_string());
		}
		match size {
			Some(s) => if let Err(e) = MilkTank::check_capacity(s) {
				return Err(e.to_string());
			},
			None => (),
		}
		Ok(())
	}
}
impl ExecJob for MilkTank {
	fn exec_job(timeout: usize, size: Option<Size>) -> Result<(), String> {
		MilkTank::dispense(timeout, size)
	}
}

pub struct Frother;
impl Ping for Frother {
	fn ping(timeout: usize) -> Result<(), String> {
		let rng = thread_rng().gen_range(2..100);
		thread::sleep(time::Duration::from_millis(rng));
		if rng as usize > timeout {
			Err("Frother Component Not Responding".to_string())
		} else {
			Ok(())
		}
	}
}
impl Frother {
	fn froth(timeout: usize) -> Result<(), String> {
		if let Err(e) = Frother::ping(timeout) {
			return Err(e.to_string());
		}
		Ok(())
	}
}
impl ExecJob for Frother {
	fn exec_job(timeout: usize, _: Option<Size>) -> Result<(), String> {
		Frother::froth(timeout)
	}
}