use dbus::blocking::Connection;
use dbus::strings::Path;

use pulse::callbacks::ListResult;
use pulse::context::Context;
use pulse::mainloop::standard::Mainloop;

use std::cell::Cell;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub fn start() -> mpsc::Sender<Path<'static>> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || task(rx));
    return tx;
}

fn task(rx: mpsc::Receiver<Path<'static>>) {
    let conn = Connection::new_system().expect("DBus connection");

    while let Ok(path) = rx.recv() {
        let proxy = conn.with_proxy("org.bluez", &path, Duration::from_secs(10));

        loop {
            match proxy.method_call("org.bluez.Device1", "Connect", ()) {
                Err(e) => {
                    if e.message() == Some("In Progress") {
                        thread::sleep(Duration::from_secs(1));
                    } else {
                        eprintln!("Failed to connect to {}: {}", path, e);
                        break;
                    }
                }
                Ok(()) => {
                    set_card_profile(path);
                    break;
                }
            }
        }
    }

    todo!()
}

fn set_card_profile(path: Path) {
    // Connect to PulseAudio and prepare an Introspector instance
    let mut mainloop = Mainloop::new().expect("Connect to PulseAudio");
    let mut context = Context::new(&mainloop, "headsets-bluez").unwrap();

    context
        .connect(None, pulse::context::flags::NOFLAGS, None)
        .expect("Failed to connect context");

    loop {
        if !mainloop.iterate(false).is_success() {
            panic!("mainloop.iterate failed");
        }

        match context.get_state() {
            pulse::context::State::Ready => {
                break;
            }
            pulse::context::State::Failed | pulse::context::State::Terminated => {
                panic!("PulseAudio context failed");
            }
            _ => thread::sleep(Duration::from_millis(100)),
        }
    }

    let mut introspect = context.introspect();

    // Find card for the new device

    #[derive(Clone, Copy, PartialEq)]
    enum State {
        Empty,
        Card(u32),
        SetProfile,
        Completed,
    }

    let state = Rc::new(Cell::new(State::Empty));

    let path_str = path.as_cstr().to_string_lossy().to_string();
    let state2 = state.clone();

    introspect.get_card_info_list(move |lr| match lr {
        ListResult::Item(card) => {
            if card.proplist.get_str("bluez.path").as_ref() == Some(&path_str) {
                println!("set-profile: card {} ({:?})", card.index, card.name);
                state2.set(State::Card(card.index));
            }
        }

        _ => {
            if state2.get() == State::Empty {
                state2.set(State::Completed)
            }
        }
    });

    while mainloop.iterate(true).is_success() {
        if let State::Card(card_index) = state.get() {
            let state = state.clone();
            state.set(State::SetProfile);
            introspect.set_card_profile_by_index(
                card_index,
                "a2dp_sink",
                Some(Box::new(move |success| {
                    println!(
                        "set-profile: {}",
                        if success { "success" } else { "failed" }
                    );

                    state.set(State::Completed);
                })),
            );
        }

        if state.get() == State::Completed {
            break;
        }
    }
}
