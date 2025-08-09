use chrono::prelude::*;
use serde::{Deserialize, Serialize};

/// Core Data type
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DataType {
    /// Electronic messages between users. Gmail messages, Outlook emails
    Email,
    /// Short-form text communication. SMS, WhatsApp, Telegram, Slack messages
    Message,
    /// Scheduled events and reminders. Google Calendar events, Outlook calendar entries
    Calendar,
    /// Information about people or organizations. Names, emails, phone numbers, social profiles, location
    Contract,
    /// User-written notes or memos. Evernote, Apple Notes, Notion pages, plain text notes
    Note,
    /// To-do items or actionable tasks. Task list entries, project management tasks (Trello, Asana)
    Task,
    /// Structured files containing text or data. PDFs, Word documents, Google Docs, spreadsheets
    Document,
    /// Static visual content. Photos, screenshots, infographics
    Image,
    /// Moving visual content with or without audio. Recorded meetings, short-form social videos
    Video,
    /// Non-transcribed sound data. Voice memos, podcasts, raw call recordings
    Audio,
    /// Published social media or blog content, Tweets, LinkedIn posts, blog articles
    Post,
    /// Financial or bill. Bank transactions, crypto wallet transfers, blockchain events
    Bill,
}

/// Core Data struct.
/// Example.
/// {
///     "source": "gmail",
///     "type": "email",
///     "title": "Meeting with John",
///     "content": "...",
///     "timestamp": "2025-08-05T09:00:00Z",
///     "tags": ["meeting", "projectX"]
/// }
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Data {
    r#type: DataType,
    source: String,
    title: String,
    content: String,
    timestamp: NaiveDateTime,
    tags: Vec<String>,
}
