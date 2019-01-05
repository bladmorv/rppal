// Copyright (c) 2017-2019 Rene van der Meer
//
// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL
// THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

// gpio_blinkled_signals.rs
//
// Blinks an LED attached to a GPIO pin in a loop, while handling any incoming
// SIGINT (Ctrl-C) and SIGTERM signals so the pin's state can be reset before
// the application exits.
//
// Remember to add a resistor of an appropriate value in series, to prevent
// exceeding the maximum current rating of the GPIO pin and the LED.

use std::process::exit;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

// The simple-signal crate is used to handle incoming signals.
use simple_signal::{set_handler, Signal};

use rppal::gpio::Gpio;

// Gpio uses BCM pin numbering. BCM GPIO 23 is tied to physical pin 16.
const GPIO_LED: u8 = 23;

fn main() {
    let gpio = Gpio::new().unwrap_or_else(|e| {
        eprintln!("Error: Can't access GPIO peripheral ({})", e);
        exit(1);
    });

    // Retrieve the GPIO pin and configure it as an output.
    let mut pin = gpio
        .get(GPIO_LED)
        .unwrap_or_else(|| {
            eprintln!("Error: Can't access GPIO pin {}", GPIO_LED);
            exit(1);
        })
        .into_output();

    // Clone running, so we can safely access it from within the signal handler.
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // When a SIGINT (Ctrl-C) or SIGTERM signal is caught, atomically set
    // running to false.
    set_handler(&[Signal::Int, Signal::Term], move |_| {
        r.store(false, Ordering::SeqCst);
    });

    // Blink the LED until running is set to false.
    while running.load(Ordering::SeqCst) {
        pin.toggle();
        sleep(Duration::from_millis(500));
    }

    // After we're done blinking, turn the LED off.
    pin.set_low();

    // When the pin variable goes out of scope at the end of the main()
    // function, the GPIO pin mode is automatically reset to its original
    // value, provided reset_on_drop is set to true (default).
}
