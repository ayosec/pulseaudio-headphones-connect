use dbus::arg::{RefArg, Variant};
use dbus::blocking::Connection;
use dbus::message::{MatchRule, Message, MessageType};
use dbus::strings::{Interface, Path};

use std::collections::HashMap;
use std::sync::mpsc;
use std::time::Duration;

type DBusProps = HashMap<String, Variant<Box<dyn RefArg>>>;

const TIMEOUT: Duration = Duration::from_secs(7200);

/// Start a loop to detect when the expected device is connected
pub fn start(mac_device: &str, queue: mpsc::Sender<Path<'static>>) -> Result<(), dbus::Error> {
    // TODO: check for others /org/bluez/hci*
    let path = {
        let mut path = mac_device.replace(":", "_");
        path.insert_str(0, "/org/bluez/hci0/dev_");
        Path::new(path).unwrap()
    };

    // Connect to DBus and add a rule to receive messages
    let mut conn = Connection::new_system()?;

    let mut rule = MatchRule::new();
    rule.msg_type = Some(MessageType::Signal);
    rule.interface = Some(Interface::from("org.freedesktop.DBus.Properties"));

    conn.add_match(rule, move |_: (), _, msg| handle(msg, &path, &queue))?;

    // Infinite loop to wait for messages
    loop {
        conn.process(TIMEOUT)?;
    }
}

/// Handle the message sent by DBus. Always returns `true`, so the match rule
/// is never removed.
fn handle(msg: &Message, target_path: &Path, queue: &mpsc::Sender<Path>) -> bool {
    let path = match msg.path() {
        Some(p) if &p == target_path => p,
        _ => return true,
    };

    let (iface, props): (_, Option<DBusProps>) = msg.get2();

    if iface
        .unwrap_or(String::new())
        .starts_with("org.bluez.Device")
    {
        let connected = props
            .and_then(|p| p.get("Connected").and_then(|v| v.as_u64()))
            .unwrap_or(0);

        if connected == 1 {
            queue.send(path.into_static()).unwrap();
        }
    }

    true
}
