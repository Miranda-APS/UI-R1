/// Web UI — Dashboard interattiva per Prometeo.
///
/// Serve una singola pagina HTML con Canvas, WebSocket e REST API.
/// L'engine vive in un thread OS dedicato (non e Send).

pub mod state;
pub mod api;
pub mod ws;
pub mod server;
pub mod conversations;
