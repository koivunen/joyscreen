// Copyright 2016-2018 Mateusz Sieczko and other GilRs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

extern crate env_logger;
extern crate gilrs;
extern crate ta;
extern crate chrono;
use chrono::prelude::*;

extern crate xcb;
use gilrs::GilrsBuilder;
use gilrs::ev::filter::{Filter, Repeat};

use std::process;
use std::thread;
use std::time::{Duration, Instant};

//use std::time::{Duration, SystemTime};
//use ta::indicators::ExponentialMovingAverage;

use ta::indicators::SimpleMovingAverage;
use ta::Next;

#[macro_use]
extern crate log;

use std::env;
use log::{Record, LevelFilter};
use env_logger::Builder;
use chrono::Local;
use std::io::Write;





struct Monitors {
    last: bool,
	offd: Duration,
	want: bool,
	ts_off: std::time::Instant,
}

impl Monitors {
	fn should_turn_off(&mut self) -> bool {
		if self.want==false && self.last==true {
			return self.can_turn_off();
		}
		return false;
	}
	
	fn can_turn_off(&mut self) -> bool {
		let now = Instant::now();
		return now - self.ts_off > self.offd;
	}
	
    fn think(&mut self) {
		if self.should_turn_off() {
			info!("Delay triggler");
			self.toggle(false);
		};
	}
	
    fn toggle(&mut self, set_on: bool) -> bool {
		
		// always set what we want
		self.want = set_on;
		
		// if it is what we have, give up
		if self.want == self.last { return self.want; };
		
		// turning off is special: once every N seconds. It will happen eventually: think func.
		if !self.want && !self.can_turn_off() {
			return self.want;
		};
		
		// we are setting off, set timestamp
		if !self.want {
			self.ts_off = Instant::now();
		};
		
		info!("Turning monitor: {}", self.want);
		
		let (conn, _) = xcb::Connection::connect(None)
			.expect("Failed to connect");
		
		let mode = if self.want { xcb::dpms::DPMS_MODE_ON } else { xcb::dpms::DPMS_MODE_OFF };
		
		xcb::dpms::enable(&conn);
		
		xcb::dpms::set_timeouts(&conn,0,0,0);
		
		xcb::dpms::force_level(&conn, mode as u16)
			.request_check()
			.expect("Failed to turn off monitor");
		
		
		// last is now set
		self.last = self.want;
		
		self.last
    }
	
    fn new() -> Monitors {
        Monitors { 
			last: false, 
			want: false,
			offd: Duration::new(10, 0),
			ts_off: Instant::now() - Duration::from_secs(60*60*2)
		}
	}
}


fn main() {
	
    let mut builder = Builder::from_default_env();

    builder.format(|buf, record| writeln!(buf,"{} [{}] - {}",
				Local::now().format("%Y-%m-%dT%H:%M:%S"),
				record.level(),
				record.args()))
				
           .filter(None, LevelFilter::Debug);
		   
		   
    if env::var("RUST_LOG").is_ok() {
       builder.parse(&env::var("RUST_LOG").unwrap());
    }
	builder.init();

	info!("Starting!");
	
    let mut monitors = Monitors::new();
	monitors.toggle(true);
	

	let mut gilrs = match GilrsBuilder::new().set_update_state(false).build() {
		Ok(g) => g,
		Err(gilrs::Error::NotImplemented(g)) => {
			eprintln!("Current platform is not supported");

			g
		}
		Err(e) => {
			eprintln!("Failed to create gilrs context: {}", e);
			process::exit(-1);
		}
	};

	let repeat_filter = Repeat::new();

	let mut sma = SimpleMovingAverage::new(30).unwrap();

	info!("Entering monitoring!");
	
	loop {
		while let Some(ev) = gilrs.next_event().filter_ev(&repeat_filter, &mut gilrs) {
			gilrs.update(&ev);
						
			match ev.event {
				gilrs::ev::EventType::AxisChanged(a,b,c) => {
					let val = sma.next(b as f64);
					
					monitors.toggle(val > -0.05);
					
					debug!("{:?}", val)
				},
				_    =>  {},
			}
			
			//if (ev.event == gilrs::ev::EventType::AxisChanged) {
			//	println!("{:?}", ev);
			//}
			monitors.think();
		}
		monitors.think();
		gilrs.inc();
		thread::sleep(Duration::from_millis(1));
	}
}

