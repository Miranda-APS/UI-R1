/// ConversationStore — Chat storicizzate inter-sessione.
///
/// Due tipi di conversazioni:
///   - Iniziate dall'utente: standard chat, persistita
///   - Iniziate dall'entità: UI-r1 ha qualcosa da dire, appare come "non letto"
///
/// Persistenza: conversations.json (separato dal .bin cognitivo)

use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

fn new_id() -> String {
    let t = now_secs();
    let r: u32 = rand_u32();
    format!("{:x}{:08x}", t, r)
}

fn rand_u32() -> u32 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    now_secs().hash(&mut h);
    std::thread::current().id().hash(&mut h);
    h.finish() as u32
}

// ─── Tipi ─────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Initiator {
    User,
    Entity,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Sender {
    User,
    Entity,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub id: String,
    pub sender: Sender,
    pub content: String,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub initiator: Initiator,
    pub created_at: u64,
    pub last_message_at: u64,
    /// True = l'utente non ha ancora aperto questa conversazione
    pub unread: bool,
    pub messages: Vec<Message>,
}

impl Conversation {
    /// Genera un titolo dalle prime ~6 parole del messaggio di apertura.
    fn title_from(text: &str) -> String {
        let words: Vec<&str> = text.split_whitespace().take(6).collect();
        let t = words.join(" ");
        if t.len() > 50 { format!("{}…", &t[..47]) } else { t }
    }

    pub fn new_user(first_message: String) -> Self {
        let now = now_secs();
        let title = Self::title_from(&first_message);
        Self {
            id: new_id(),
            title,
            initiator: Initiator::User,
            created_at: now,
            last_message_at: now,
            unread: false,
            messages: vec![Message {
                id: new_id(),
                sender: Sender::User,
                content: first_message,
                timestamp: now,
            }],
        }
    }

    pub fn new_entity(opening: String) -> Self {
        let now = now_secs();
        let title = Self::title_from(&opening);
        Self {
            id: new_id(),
            title,
            initiator: Initiator::Entity,
            created_at: now,
            last_message_at: now,
            unread: true,
            messages: vec![Message {
                id: new_id(),
                sender: Sender::Entity,
                content: opening,
                timestamp: now,
            }],
        }
    }

    pub fn add_entity_reply(&mut self, content: String) {
        let now = now_secs();
        self.last_message_at = now;
        self.messages.push(Message {
            id: new_id(),
            sender: Sender::Entity,
            content,
            timestamp: now,
        });
    }

    pub fn add_user_reply(&mut self, content: String) {
        let now = now_secs();
        self.last_message_at = now;
        self.unread = false;
        self.messages.push(Message {
            id: new_id(),
            sender: Sender::User,
            content,
            timestamp: now,
        });
    }
}

// ─── Store ────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Default)]
pub struct ConversationStore {
    pub conversations: Vec<Conversation>,
    /// Tick (seconds) dell'ultima conversazione aperta dall'entità.
    /// Evita che l'entità apra decine di chat in rapida successione.
    pub last_entity_conv_at: u64,
}

const CONV_PATH: &str = "conversations.json";
/// Intervallo minimo tra due conversazioni aperte dall'entità (30 minuti)
const ENTITY_CONV_COOLDOWN: u64 = 1800;

impl ConversationStore {
    pub fn load_or_new() -> Self {
        if let Ok(data) = std::fs::read_to_string(CONV_PATH) {
            if let Ok(store) = serde_json::from_str(&data) {
                return store;
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(CONV_PATH, json);
        }
    }

    // ─── Accesso ──────────────────────────────────────────────────────────

    pub fn get(&self, id: &str) -> Option<&Conversation> {
        self.conversations.iter().find(|c| c.id == id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut Conversation> {
        self.conversations.iter_mut().find(|c| c.id == id)
    }

    pub fn unread_count(&self) -> usize {
        self.conversations.iter().filter(|c| c.unread).count()
    }

    // ─── Mutazioni ────────────────────────────────────────────────────────

    /// Crea nuova conversazione iniziata dall'utente.
    /// Restituisce l'id della nuova conversazione.
    pub fn start_user_conversation(&mut self, first_message: String) -> String {
        let conv = Conversation::new_user(first_message);
        let id = conv.id.clone();
        self.conversations.insert(0, conv);
        self.save();
        id
    }

    /// Aggiunge risposta dell'entità a una conversazione esistente.
    pub fn add_entity_reply(&mut self, conv_id: &str, content: String) {
        if let Some(conv) = self.get_mut(conv_id) {
            conv.add_entity_reply(content);
        }
        self.save();
    }

    /// Aggiunge risposta dell'utente a una conversazione esistente.
    pub fn add_user_reply(&mut self, conv_id: &str, content: String) {
        if let Some(conv) = self.get_mut(conv_id) {
            conv.add_user_reply(content);
        }
        self.save();
    }

    /// Segna conversazione come letta.
    pub fn mark_read(&mut self, conv_id: &str) {
        if let Some(conv) = self.get_mut(conv_id) {
            conv.unread = false;
        }
        self.save();
    }

    /// Apre una nuova conversazione dall'entità.
    /// Rispetta il cooldown: se troppo recente, non fa nulla.
    /// Restituisce Some(id) se la conversazione è stata creata.
    pub fn try_start_entity_conversation(&mut self, opening: String) -> Option<String> {
        let now = now_secs();
        if now - self.last_entity_conv_at < ENTITY_CONV_COOLDOWN {
            return None;
        }
        let conv = Conversation::new_entity(opening);
        let id = conv.id.clone();
        self.conversations.insert(0, conv);
        self.last_entity_conv_at = now;
        self.save();
        Some(id)
    }

    /// Elimina una conversazione per id.
    pub fn delete(&mut self, conv_id: &str) {
        self.conversations.retain(|c| c.id != conv_id);
        self.save();
    }
}

// ─── DTO per la UI ────────────────────────────────────────────────────────

#[derive(Serialize, Clone)]
pub struct ConvSummaryDto {
    pub id: String,
    pub title: String,
    pub initiator: String,     // "user" | "entity"
    pub last_message_at: u64,
    pub unread: bool,
    pub message_count: usize,
    pub preview: String,       // ultimo messaggio troncato
}

impl ConvSummaryDto {
    pub fn from(conv: &Conversation) -> Self {
        let preview = conv.messages.last()
            .map(|m| {
                let t = &m.content;
                if t.len() > 80 { format!("{}…", &t[..77]) } else { t.clone() }
            })
            .unwrap_or_default();
        Self {
            id: conv.id.clone(),
            title: conv.title.clone(),
            initiator: match conv.initiator {
                Initiator::User => "user".to_string(),
                Initiator::Entity => "entity".to_string(),
            },
            last_message_at: conv.last_message_at,
            unread: conv.unread,
            message_count: conv.messages.len(),
            preview,
        }
    }
}

#[derive(Serialize, Clone)]
pub struct MessageDto {
    pub id: String,
    pub sender: String,    // "user" | "entity"
    pub content: String,
    pub timestamp: u64,
}

impl MessageDto {
    pub fn from(m: &Message) -> Self {
        Self {
            id: m.id.clone(),
            sender: match m.sender {
                Sender::User => "user".to_string(),
                Sender::Entity => "entity".to_string(),
            },
            content: m.content.clone(),
            timestamp: m.timestamp,
        }
    }
}

#[derive(Serialize, Clone)]
pub struct ConvFullDto {
    pub id: String,
    pub title: String,
    pub initiator: String,
    pub created_at: u64,
    pub unread: bool,
    pub messages: Vec<MessageDto>,
}

impl ConvFullDto {
    pub fn from(conv: &Conversation) -> Self {
        Self {
            id: conv.id.clone(),
            title: conv.title.clone(),
            initiator: match conv.initiator {
                Initiator::User => "user".to_string(),
                Initiator::Entity => "entity".to_string(),
            },
            created_at: conv.created_at,
            unread: conv.unread,
            messages: conv.messages.iter().map(MessageDto::from).collect(),
        }
    }
}

#[derive(Serialize, Clone)]
pub struct ConvListDto {
    pub conversations: Vec<ConvSummaryDto>,
    pub unread_count: usize,
}
