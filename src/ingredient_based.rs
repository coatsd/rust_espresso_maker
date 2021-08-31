use std::{time, thread};
use std::sync::mpsc;
use std::fmt;

type S<T> = mpsc::Sender<T>;
type R<T> = mpsc::Receiver<T>;
const ROOM_TEMP: f32 = 70.0;
const FRIDGE_TEMP: f32 = 42.0;

struct CoffeeBeans { weight: f32 }
impl fmt::Display for CoffeeBeans {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} oz. of Coffee Beans", self.weight)
    }
}

struct CoffeeGrounds { weight: f32 }
impl fmt::Display for CoffeeGrounds {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} oz of Coffee Grounds", self.weight)
    }
}

struct Water { weight: f32, temp: f32 }
impl fmt::Display for Water {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} oz of Water, temperature: {}", self.weight, self.temp)
    }
}

struct Milk { weight: f32, temp: f32 }
impl fmt::Display for Milk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} oz of Milk, temperature: {}", self.weight, self.temp)
    }
}

struct Espresso { weight: f32, temp: f32 }
impl fmt::Display for Espresso {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} oz of Espresso, temperature: {}", self.weight, self.temp)
    }
}

struct Latte { weight: f32, temp: f32 }
impl fmt::Display for Latte {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} oz of Latte, temperature: {}", self.weight, self.temp)
    }
}

fn grind_beans(cb: CoffeeBeans, cg_send: S<CoffeeGrounds>) {
    thread::sleep(time::Duration::from_millis(500));
    let cg = CoffeeGrounds { weight: cb.weight };
    println!("Made {}!", cg);
    match cg_send.send(cg) {
        Result::Err(e) => println!("Error in cg_send: {}", e),
        _ => println!("Coffee beans ground!"),
    }
    drop(cg_send);
}

fn heat_water(mut w: Water, w_send: S<Water>) {
    thread::sleep(time::Duration::from_millis(1000));
    w.temp = 185.0;
    println!("Heated {}!", w);
    match w_send.send(w) {
        Result::Err(e) => println!("Error in w_send: {}", e),
        _ => println!("Water heated!"),
    }
    drop(w_send);
}

fn heat_milk(mut m: Milk, m_send: S<Milk>) {
    thread::sleep(time::Duration::from_millis(750));
    m.temp = 150.0;
    println!("Heated {}!", m);
    match m_send.send(m) {
        Result::Err(e) => println!("Error in m_send: {}", e),
        _ => println!("Milk heated!"),
    }
    drop(m_send);
}

fn press_espresso(cg_recv: R<CoffeeGrounds>, w_recv: R<Water>, e_send: S<Espresso>) {
    thread::sleep(time::Duration::from_millis(500));
    let mut e = Espresso { weight: 0.0, temp: 0.0 };
    let cg = cg_recv.recv();
    let w = w_recv.recv();
    if cg.is_ok() {
        if w.is_ok() {
            let w = w.unwrap();
            e.weight = w.weight;
            e.temp = w.temp;
            match e_send.send(e) {
                Result::Err(e) => println!("Error in e_send: {}", e),
                _ => println!("Espresso pressed!"),
            }
        }
        else {
            println!("Failed to get water!");
        }
    }
    else {
        println!("Failed to get coffee grounds!");
    }
    drop(e_send);
}

fn froth_milk(m_recv: R<Milk>, m_send: S<Milk>) {
    thread::sleep(time::Duration::from_millis(500));
    let m = m_recv.recv().unwrap();
    match m_send.send(m) {
        Result::Err(e) => println!("Error in m_send: {}", e),
        _ => println!("Milked frothed!"),
    }
    drop(m_send);
}

fn make_latte(m_recv: R<Milk>, e_recv: R<Espresso>) {
    if let Result::Ok(_) = e_recv.recv() {
        if let Result::Ok(_) = m_recv.recv() {
            thread::sleep(time::Duration::from_millis(250));
            println!("Enjoy your latte!");
        }
    }
}

pub fn ingredient_based_main() {
    let water = Water { weight: 2.0, temp: ROOM_TEMP };
    let milk = Milk { weight: 6.0, temp: FRIDGE_TEMP };
    let coffee_beans = CoffeeBeans { weight: 1.0 };

    use mpsc::channel;

    let (cg_send1, cg_recv1) = channel::<CoffeeGrounds>();
    let (w_send1, w_recv1) = channel::<Water>();
    let (m_send1, m_recv1) = channel::<Milk>();
    let (e_send1, e_recv1) = channel::<Espresso>();
    let (m_send2, m_recv2) = channel::<Milk>();

    let threads = vec![
        thread::spawn(move || grind_beans(coffee_beans, cg_send1)),
        thread::spawn(move || heat_water(water, w_send1)),
        thread::spawn(move || heat_milk(milk, m_send1)),
        thread::spawn(move || press_espresso(cg_recv1, w_recv1, e_send1)),
        thread::spawn(move || froth_milk(m_recv1, m_send2)),
    ];

    for t in threads {
        if let Result::Err(e) = t.join() {
            if let Some(e) = e.downcast_ref::<&'static str>() {
                println!("Error: {}", e);
            } else {
                println!("Unknown Error: {:?}", e);
            }
        }
    }

    make_latte(m_recv2, e_recv1);
}