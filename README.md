# VoltaAlert

Tauri application that listens to a ZeroMQ stream, stores events in SQLite, and notifies the frontend in real time.

## Architecture

I decided to split in 3 crate the application and allow distinct responsability for each crate

| Crate | Name | Role |
|-------|------|------|
| `core` | `voltaalert-core` | Library: domain types, SQLite repo, service & manager logic |
| `clock` | `voltaalert-clock` | CLI: ZeroMQ publisher / subscriber for testing |
| `voltaalert` | Tauri app | Desktop UI, using service and manager from core crate |

So, starting with clock project you can mock, simulate and publish datas to ZeroMQ.<br/>
ZeroMQ will be the communication layer between clock and the app

Tauri app will have 2 workers:
- **Clock -> AlertService**: Handle datas from **clock**, then generate alerts from AlertSevice (with AlarmManager)
- **Rust -> Frontend**: to emit event when alerts are emitted

**AlertService** manage the orchestrate the alerting logic<br/>
**AlarmManager** handle alarm repo, run alarm trigger evalution that fire alerts

## Clock

CLI to drive the ZeroMQ bus.<br/>

```bash
# Publisher (default: tcp://127.0.0.1:9000, 1 msg/sec)
cargo run -p voltaalert-clock

# Subscriber — print received messages
cargo run -p voltaalert-clock -- --subscribe

# Custom address
cargo run -p voltaalert-clock -- --port 9001 --addr 0.0.0.0
```

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--port` | `-p` | `9000` | TCP port |
| `--addr` | `-a` | `127.0.0.1` | Bind / connect address |
| `--protocol` | 𐄂 | `tcp` | ZeroMQ transport |
| `--subscribe` | `-s` | `false` | Run as subscriber instead of publisher |
| `--mode` | `-m` | `period` | see below |


Publisher mode has different modes for input generation behavior:
| Mode | Description |
|------|-------------|
| `period` | Default behavior: push one a timestamp every seconds |
| `file` | Read datas from a file (require `--file=<path>` to work). For random data on UNIX, use `--file=/dev/urandom` |
| `manual` | Read lines from stdin |

## Core

### Alarm & AlarmManager

Each alarm consist in following :
- **`alarm::Alarm`** - trait implementation (logic for fire an alert)
- **`alarm::AlarmDescriptor`** - datastore representation
- **`alarm::AlarmKind`** - enum of available kinds of alarms (each value implements **`alarm::Alarm`**)

Actually there is 3 kinds :
- **`Clock`** a simple alarm clock (wake up !), it can trigger only one time an alert
- **`Pattern`** a comparison based trigger (equals / different), could be extended
- **`Always`** kind of debug alarm; it will always decide to fire an alert, even when already emitted

There are other important stuff:
- **`repo::AlarmRepository`** contains basics structs and traits for CRUD operations on AlarmDescription
- **`repo::sqlite::SqliteAlarmRepository`** Sqlite implementation of AlarmRepository

**`alarm::manager::AlarmManager`**  is the executor for alarms
It keeps in memory all active alarms, loaded when instanciate a manager.<br>
When an alert is create/modified/deleted, manager will reflect changes in active alarm and in repository.

`alarm::manager::AlarmManager::evaluate_alarms` is the most important method; it take a raw byte data and check for every active alarm if it must generate an alert (and optionally modified from states: .i.e. trigger once for **`AlarmClock`**)

### Alerts & AlertService

- **`types::Alert`** represents a data pushed by **`clock`** that trigger a Alarm.
- **`repo::AlertRepository`** — CRUD + pagination trait; SQLite implementation in `repo::sqlite`
- **`service::AlertService`** — Logic handler

**`service::AlertService`** is not communicating directly with ZeroMq, instead there is a channel (`tokio::sync::mpsc`) for this.
That channel is created on AlertService instanciation.<br/>
```rust
let (service, alert_recv_rx) = service::AlertService::new(...)
```
It also delegate CRUD actions on alerts (and alarms) to repositories<br/>
`service::AlertService::evaluate_received_data(Vec<u8>)` will run alarm manager logic, store alerts and push it to the channel

## Voltaalert (Tauri)

The setup consist of:
- Create hooks for log crate (note: I had twice log message in stdout by setting a target -> TODO!)
- Prepare service
- Spawn workers

## Getting started

**Prerequisites:** Rust, Node.js, ZeroMQ system library

```bash
# Frontend + Tauri dev mode
cd voltaalert
npm install
npm run tauri dev
```

```bash
# In a separate terminal — start the ZMQ publisher
cargo run -p voltaalert-clock
```

Note: The app connects to `tcp://127.0.0.1:9000`

## Data flow

1. `clock` publishes raw bytes on ZeroMQ
2. `ZeromqClient::recv()` delivers them to the `AlertManager` by using `AlertService::evaluate_received_data()`
3. On match: alert is persisted in SQLite, then sent on an `mpsc::Sender`
4. A background task reads the channel and calls `app_handle.emit("alert", alert_data)`
5. The React frontend receives the alert via `listen("alert", ...)`
6. Enjoy