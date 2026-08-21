# Acton Core

Кристаллический протокол связи. Сообщения кодируются в координаты 3D-массива, передаются без ACK, без установки соединения.

## Особенности

- Нет серверов
- Нет метаданных
- Работает поверх любого транспорта
- Шифрование + теги

## Пример

```rust
use acton_core::{ActonClient, MemoryTransport};

let mut alice = ActonClient::new("seed1", Box::new(MemoryTransport::new("alice")));
let mut bob = ActonClient::new("seed2", Box::new(MemoryTransport::new("bob")));

let session = alice.initiate_session(&bob.identity().public_id)?;
alice.send(&session.session_id(), b"Hello")?;

let (_, msg) = bob.receive_blocking()?;
println!("{}", String::from_utf8_lossy(&msg));