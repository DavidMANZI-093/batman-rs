# batman-rs

Battery manager daemon for Linux. Monitors hardware power events and executes
user-defined shell commands at any battery level or power state transition.


## Design

batman opens a raw `AF_NETLINK` socket subscribed to `NETLINK_KOBJECT_UEVENT` and
blocks waiting for the kernel to deliver power supply events. When the kernel
reports a change: a charger being plugged in, a battery capacity tick, a
status transition; batman wakes, parses the event, evaluates the configured
rules, and executes any matching commands via the shell.

The process is otherwise idle. No polling loop. No timer. No periodic wakeups.
Resource usage is negligible between events.


## Dependencies

    systemd               - systemctl        (power-off, assumed system default)
    libnotify             - notify-send      (desktop notifications)
    power-profiles-daemon - powerprofilesctl (power profile switching)

`libnotify` and `power-profiles-daemon` are optional but recommended. The sample
configuration is built around them.


## Installation

### Arch Linux

#### AUR (recommended)

Install using an AUR helper such as yay or paru:

```bash
yay -S batman-rs
```

A default configuration is installed to `/etc/batman/config.toml`. Edit it to match your setup before enabling the service.

#### From source

Requires a stable Rust toolchain:

```bash
git clone https://github.com/d3fault/batman-rs
cd batman-rs
cargo build --release
install -Dm755 target/release/batman-rs ~/.local/bin/batman
install -Dm644 batman.service ~/.config/systemd/user/batman.service
mkdir -p ~/.config/batman
cp config.toml.sample ~/.config/batman/config.toml
```

### Other Linux Distributions

Build the binary with Cargo:

```bash
git clone https://github.com/d3fault/batman-rs
cd batman-rs
cargo build --release
```

Then install the binary to a location on your `PATH` and set up the service using
the appropriate file from the `contrib/` directory. See Service Management below.


## Configuration

batman searches for a configuration file in the following order:

    1. Path given by --config <FILE>
    2. $XDG_CONFIG_HOME/batman/config.toml
    3. ~/.config/batman/config.toml
    4. /etc/batman/config.toml

On Arch Linux (AUR), a default configuration is installed to `/etc/batman/config.toml`.
Edit it directly, or copy it to your user config directory for per-user overrides:

```bash
mkdir -p ~/.config/batman
cp /etc/batman/config.toml ~/.config/batman/config.toml
```

On other distributions, copy the sample from the source tree:

```bash
mkdir -p ~/.config/batman
cp config.toml.sample ~/.config/batman/config.toml
```


### Rule Fields

    state         : Charging | Discharging | Full | NotCharging | AcOnline | AcOffline
    capacity_under: (optional) fires only on a downward threshold crossing
    command       : shell command - supports &&, ;, ||, pipes, and redirections

Example:

```toml
[[rules]]
state = "Discharging"
capacity_under = 10
command = "notify-send -u critical 'Battery Critical' 'Battery at 10%'"
```

### Firing Semantics

Rules without `capacity_under` fire on every matching kernel uevent. Understanding
when the kernel sends these events is essential for writing correct rules.

AcOnline / AcOffline

    The kernel sends one event per physical plug or unplug of an AC adapter.
    Safe to use for any command, including stateful ones such as switching
    power profiles.

Charging

    The kernel sends a battery uevent on every capacity change while charging,
    approximately once per percent. A bare Charging rule without capacity_under
    will fire roughly 100 times during a full charge cycle. Only use idempotent
    commands in this state.

Full / NotCharging

    The kernel sends periodic uevents while the battery holds these states.
    Frequency varies by hardware. Use only idempotent commands.

Discharging + capacity_under

    The rule fires exactly once when battery capacity crosses downward below or
    equal to the threshold value. This is the recommended pattern for all
    battery alerts and threshold-triggered actions.


## Service Management

batman must run in the user session context so that notify-send and D-Bus
services such as powerprofilesctl are reachable without additional setup.

### systemd

batman ships a user service unit. Enable and start it with:

```bash
systemctl --user enable --now batman
```

Other commands:

```bash
systemctl --user status batman
systemctl --user stop batman
journalctl --user -u batman -f
```

### OpenRC

A service script and conf.d template are provided in `contrib/`:

```bash
sudo install -Dm755 contrib/batman.openrc /etc/init.d/batman
sudo install -Dm644 contrib/batman.conf   /etc/conf.d/batman
```

Edit `/etc/conf.d/batman` and set `BATMAN_USER` to your login username. Then enable
the service:

```bash
sudo rc-update add batman default
sudo rc-service batman start
```

This requires elogind (or systemd-logind) to be running so that the user D-Bus
session is available at `/run/user/<UID>/bus`.

### runit

A run script is provided in `contrib/batman.runit` for use as a per-user service.
On Void Linux:

```bash
mkdir -p ~/.local/share/sv/batman
cp contrib/batman.runit ~/.local/share/sv/batman/run
chmod +x ~/.local/share/sv/batman/run
ln -s ~/.local/share/sv/batman ~/.local/share/sv/batman
```

Refer to the Void Linux handbook for enabling per-user runsvdir.

### Other / Login session

If none of the above apply, batman can be started directly from the user login
session. Add the following to `~/.bash_profile`, `~/.zprofile`, or the equivalent
for your shell:

```bash
/usr/bin/batman &
```


## License

GPLv3. See LICENSE.
