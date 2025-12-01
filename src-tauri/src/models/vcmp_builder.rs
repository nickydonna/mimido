use crate::{
    calendar_items::{
        event_status::EventStatus, event_type::EventType, event_upsert::EventUpsertInfo,
    },
    models::{
        NewVCmp, VCmp,
        vevent::{NewVEvent, VEvent},
        vtodo::{NewVTodo, VTodo},
    },
};
use chrono::{DateTime, TimeZone, Utc};

#[derive(Debug, Clone, Default)]
pub struct VCmpBuilder {
    calendar_url: Option<String>,
    // Required for NewVCmp only
    calendar_id: Option<i32>,
    uid: Option<String>,

    // Optional for both
    id: Option<i32>,
    href: Option<String>,
    ical_data: Option<String>,
    summary: Option<String>,
    description: Option<String>,
    starts_at: Option<DateTime<Utc>>,
    ends_at: Option<DateTime<Utc>>,
    has_rrule: bool,
    rrule_str: Option<String>,
    tag: Option<String>,
    status: Option<EventStatus>,
    event_type: Option<EventType>,
    original_text: Option<String>,
    load: Option<i32>,
    urgency: Option<i32>,
    importance: Option<i32>,
    postponed: Option<i32>,
    last_modified: Option<i64>,
    etag: Option<String>,
    synced_at: Option<i64>,

    // VTodo specific
    completed: Option<DateTime<Utc>>,
}

impl VCmpBuilder {
    /// Creates a new empty builder
    pub fn new() -> Self {
        Self::default()
    }

    pub fn calendar_href(mut self, calendar_url: impl Into<String>) -> Self {
        self.calendar_url = Some(calendar_url.into());
        self
    }

    // Builder methods

    pub fn id(mut self, id: i32) -> Self {
        self.id = Some(id);
        self
    }

    pub fn calendar_id(mut self, calendar_id: i32) -> Self {
        self.calendar_id = Some(calendar_id);
        self
    }

    pub fn uid(mut self, uid: impl Into<String>) -> Self {
        self.uid = Some(uid.into());
        self
    }

    pub fn href(mut self, href: impl Into<String>) -> Self {
        self.href = Some(href.into());
        self
    }

    pub fn ical_data(mut self, ical_data: impl Into<String>) -> Self {
        self.ical_data = Some(ical_data.into());
        self
    }

    pub fn summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = Some(summary.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn starts_at(mut self, starts_at: DateTime<Utc>) -> Self {
        self.starts_at = Some(starts_at);
        self
    }

    pub fn ends_at(mut self, ends_at: DateTime<Utc>) -> Self {
        self.ends_at = Some(ends_at);
        self
    }

    pub fn has_rrule(mut self, has_rrule: bool) -> Self {
        self.has_rrule = has_rrule;
        self
    }

    pub fn rrule_str(mut self, rrule_str: impl Into<String>) -> Self {
        self.rrule_str = Some(rrule_str.into());
        self.has_rrule = true;
        self
    }

    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = Some(tag.into());
        self
    }

    pub fn status(mut self, status: EventStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn event_type(mut self, event_type: EventType) -> Self {
        self.event_type = Some(event_type);
        self
    }

    pub fn original_text(mut self, original_text: impl Into<String>) -> Self {
        self.original_text = Some(original_text.into());
        self
    }

    pub fn load(mut self, load: i32) -> Self {
        self.load = Some(load);
        self
    }

    pub fn urgency(mut self, urgency: i32) -> Self {
        self.urgency = Some(urgency);
        self
    }

    pub fn importance(mut self, importance: i32) -> Self {
        self.importance = Some(importance);
        self
    }

    pub fn postponed(mut self, postponed: i32) -> Self {
        self.postponed = Some(postponed);
        self
    }

    pub fn last_modified(mut self, last_modified: i64) -> Self {
        self.last_modified = Some(last_modified);
        self
    }

    pub fn etag(mut self, etag: impl Into<String>) -> Self {
        self.etag = Some(etag.into());
        self
    }

    pub fn synced_at(mut self, synced_at: i64) -> Self {
        self.synced_at = Some(synced_at);
        self
    }

    pub fn completed(mut self, completed: DateTime<Utc>) -> Self {
        self.completed = Some(completed);
        self
    }

    fn get_href(&self) -> Option<String> {
        if let Some(href) = self.href.clone() {
            return Some(href);
        } else if let Some(cal_href) = self.calendar_url.clone()
            && let Some(id) = self.id
        {
            Some(format!("{cal_href}{id}.ics"))
        } else {
            None
        }
    }

    // Build methods

    /// Builds a NewVCmp from the builder
    ///
    /// The component type (Event vs Todo) is determined by:
    /// 1. If `event_type` is `EventType::Task`, creates a Todo
    /// 2. If `starts_at` is None, creates a Todo (tasks can have optional dates)
    /// 3. Otherwise creates an Event
    ///
    /// # Errors
    ///
    /// Returns an error if required fields are missing:
    /// - `calendar_id`
    /// - `uid`
    /// - `summary`
    /// - For events: `starts_at` and `ends_at`
    pub fn build_new(&self) -> anyhow::Result<NewVCmp> {
        let calendar_id = self
            .calendar_id
            .ok_or(anyhow::anyhow!("calendar_id is required"))?;
        let uid = self.uid.clone().ok_or(anyhow::anyhow!("uid is required"))?;
        let summary = self
            .summary
            .clone()
            .ok_or(anyhow::anyhow!("summary is required"))?;

        // Determine if this should be a Todo or Event
        let is_task = self
            .event_type
            .ok_or(anyhow::anyhow!("should have event_type"))?
            == EventType::Task;

        let has_no_dates = self.starts_at.is_none();
        let event_type = self.event_type.unwrap_or(EventType::Event);
        let status = self.status.unwrap_or(EventStatus::Todo);
        let load = self.load.unwrap_or(0);
        let urgency = self.urgency.unwrap_or(0);
        let importance = self.importance.unwrap_or(0);
        let postponed = self.postponed.unwrap_or(0);
        let now = Utc::now().timestamp();
        let last_modified = self.last_modified.unwrap_or(now);
        let synced_at = self.synced_at.unwrap_or(now);

        if is_task || has_no_dates {
            // Build NewVTodo
            Ok(NewVCmp::Todo(NewVTodo {
                calendar_id,
                uid,
                href: self.get_href(),
                ical_data: self.ical_data.clone(),
                summary,
                description: self.description.clone(),
                starts_at: self.starts_at,
                ends_at: self.ends_at,
                has_rrule: self.has_rrule,
                rrule_str: self.rrule_str.clone(),
                tag: self.tag.clone(),
                status,
                event_type,
                original_text: self.original_text.clone(),
                load,
                urgency,
                importance,
                postponed,
                last_modified,
                etag: self.etag.clone(),
                synced_at,
                completed: self.completed,
            }))
        } else {
            // Build NewVEvent - requires dates
            let starts_at = self
                .starts_at
                .ok_or(anyhow::anyhow!("starts_at is required for events"))?;
            let ends_at = self
                .ends_at
                .ok_or(anyhow::anyhow!("ends_at is required for events"))?;

            Ok(NewVCmp::Event(NewVEvent {
                calendar_id,
                uid,
                href: self.get_href(),
                ical_data: self.ical_data.clone(),
                summary,
                description: self.description.clone(),
                starts_at,
                ends_at,
                has_rrule: self.has_rrule,
                rrule_str: self.rrule_str.clone(),
                tag: self.tag.clone(),
                status,
                event_type,
                original_text: self.original_text.clone(),
                load,
                urgency,
                importance,
                postponed,
                last_modified,
                etag: self.etag.clone(),
                synced_at,
            }))
        }
    }

    /// Builds a VCmp from the builder
    ///
    /// Similar to `build_new` but requires an `id` field for existing database records.
    ///
    /// # Errors
    ///
    /// Returns an error if required fields are missing, including `id`
    pub fn build(&self) -> anyhow::Result<VCmp> {
        let id = self.id.ok_or(anyhow::anyhow!("id is required"))?;
        let calendar_id = self
            .calendar_id
            .ok_or(anyhow::anyhow!("calendar_id is required"))?;
        let uid = self.uid.clone().ok_or(anyhow::anyhow!("uid is required"))?;
        let summary = self
            .summary
            .clone()
            .ok_or_else(|| anyhow::anyhow!("summary is required"))?;

        // Determine if this should be a Todo or Event
        let is_task = self
            .event_type
            .ok_or(anyhow::anyhow!("should have event_type"))?
            == EventType::Task;

        let has_no_dates = self.starts_at.is_none();
        let event_type = self.event_type.unwrap_or(EventType::Event);
        let status = self.status.unwrap_or(EventStatus::Todo);
        let load = self.load.unwrap_or(0);
        let urgency = self.urgency.unwrap_or(0);
        let importance = self.importance.unwrap_or(0);
        let postponed = self.postponed.unwrap_or(0);
        let now = Utc::now().timestamp();
        let last_modified = self.last_modified.unwrap_or(now);
        let synced_at = self.synced_at.unwrap_or(now);

        if is_task || has_no_dates {
            // Build VTodo
            Ok(VCmp::Todo(VTodo {
                id,
                calendar_id,
                uid,
                href: self.get_href(),
                ical_data: self.ical_data.clone(),
                summary,
                description: self.description.clone(),
                starts_at: self.starts_at,
                ends_at: self.ends_at,
                has_rrule: self.has_rrule,
                rrule_str: self.rrule_str.clone(),
                tag: self.tag.clone(),
                status,
                event_type,
                original_text: self.original_text.clone(),
                load,
                urgency,
                importance,
                postponed,
                last_modified,
                etag: self.etag.clone(),
                synced_at,
                completed: self.completed,
            }))
        } else {
            // Build VEvent - requires dates
            let starts_at = self
                .starts_at
                .ok_or(anyhow::anyhow!("starts_at is required for events"))?;
            let ends_at = self
                .ends_at
                .ok_or(anyhow::anyhow!("ends_at is required for events"))?;

            Ok(VCmp::Event(VEvent {
                id,
                calendar_id,
                uid,
                href: self.get_href(),
                ical_data: self.ical_data.clone(),
                summary,
                description: self.description.clone(),
                starts_at,
                ends_at,
                has_rrule: self.has_rrule,
                rrule_str: self.rrule_str.clone(),
                tag: self.tag.clone(),
                status,
                event_type,
                original_text: self.original_text.clone(),
                load,
                urgency,
                importance,
                postponed,
                last_modified,
                etag: self.etag.clone(),
                synced_at,
            }))
        }
    }
}

// From trait implementations

impl From<&VEvent> for VCmpBuilder {
    fn from(event: &VEvent) -> Self {
        Self {
            calendar_url: None,
            id: Some(event.id),
            calendar_id: Some(event.calendar_id),
            uid: Some(event.uid.clone()),
            href: event.href.clone(),
            ical_data: event.ical_data.clone(),
            summary: Some(event.summary.clone()),
            description: event.description.clone(),
            starts_at: Some(event.starts_at),
            ends_at: Some(event.ends_at),
            has_rrule: event.has_rrule,
            rrule_str: event.rrule_str.clone(),
            tag: event.tag.clone(),
            status: Some(event.status),
            event_type: Some(event.event_type),
            original_text: event.original_text.clone(),
            load: Some(event.load),
            urgency: Some(event.urgency),
            importance: Some(event.importance),
            postponed: Some(event.postponed),
            last_modified: Some(event.last_modified),
            etag: event.etag.clone(),
            synced_at: Some(event.synced_at),
            completed: None,
        }
    }
}

impl From<&VTodo> for VCmpBuilder {
    fn from(todo: &VTodo) -> Self {
        Self {
            calendar_url: None,
            id: Some(todo.id),
            calendar_id: Some(todo.calendar_id),
            uid: Some(todo.uid.clone()),
            href: todo.href.clone(),
            ical_data: todo.ical_data.clone(),
            summary: Some(todo.summary.clone()),
            description: todo.description.clone(),
            starts_at: todo.starts_at,
            ends_at: todo.ends_at,
            has_rrule: todo.has_rrule,
            rrule_str: todo.rrule_str.clone(),
            tag: todo.tag.clone(),
            status: Some(todo.status),
            event_type: Some(todo.event_type),
            original_text: todo.original_text.clone(),
            load: Some(todo.load),
            urgency: Some(todo.urgency),
            importance: Some(todo.importance),
            postponed: Some(todo.postponed),
            last_modified: Some(todo.last_modified),
            etag: todo.etag.clone(),
            synced_at: Some(todo.synced_at),
            completed: todo.completed,
        }
    }
}

impl From<&VCmp> for VCmpBuilder {
    fn from(vcmp: &VCmp) -> Self {
        match vcmp {
            VCmp::Event(event) => Self::from(event),
            VCmp::Todo(todo) => Self::from(todo),
        }
    }
}

impl<Tz: TimeZone> From<&EventUpsertInfo<Tz>> for VCmpBuilder {
    fn from(info: &EventUpsertInfo<Tz>) -> Self {
        let mut builder = Self::new()
            .summary(&info.summary)
            .status(info.status)
            .event_type(info.event_type)
            .load(info.load)
            .urgency(info.urgency)
            .importance(info.importance)
            .postponed(info.postponed);

        // Handle tag
        if let Some(tag) = &info.tag.0 {
            builder = builder.tag(tag);
        }

        // Handle dates
        if let Some(date_info) = &info.date_info.0 {
            builder = builder.starts_at(date_info.start.to_utc());

            let ends_at = date_info
                .end
                .as_ref()
                .map(|e| e.to_utc())
                .unwrap_or_else(|| date_info.get_end_or_default(info.event_type).to_utc());

            builder = builder.ends_at(ends_at);

            // Handle recurrence
            if let Some(rrule_str) = date_info.clone().get_recurrence_as_cal_property() {
                builder = builder.rrule_str(rrule_str);
            }
        }

        builder
    }
}

#[cfg(test)]
mod tests {
    use crate::calendar_items::event_date::EventDateOption;

    use super::*;
    use chrono::Utc;

    #[test]
    fn test_build_new_event() {
        let now = Utc::now();
        let later = now + chrono::Duration::hours(1);

        let builder = VCmpBuilder::new()
            .calendar_id(1)
            .uid("test-uid")
            .summary("Test Event")
            .starts_at(now)
            .ends_at(later)
            .event_type(EventType::Event);

        let new_vcmp = builder.build_new().unwrap();

        match new_vcmp {
            NewVCmp::Event(event) => {
                assert_eq!(event.calendar_id, 1);
                assert_eq!(event.uid, "test-uid");
                assert_eq!(event.summary, "Test Event");
                assert_eq!(event.starts_at, now);
                assert_eq!(event.ends_at, later);
            }
            NewVCmp::Todo(_) => panic!("Expected Event, got Todo"),
        }
    }

    #[test]
    fn test_build_new_todo() {
        let builder = VCmpBuilder::new()
            .calendar_id(1)
            .uid("test-uid")
            .summary("Test Task")
            .event_type(EventType::Task);

        let new_vcmp = builder.build_new().unwrap();

        match new_vcmp {
            NewVCmp::Todo(todo) => {
                assert_eq!(todo.calendar_id, 1);
                assert_eq!(todo.uid, "test-uid");
                assert_eq!(todo.summary, "Test Task");
                assert_eq!(todo.event_type, EventType::Task);
            }
            NewVCmp::Event(_) => panic!("Expected Todo, got Event"),
        }
    }

    #[test]
    fn test_build_new_todo_without_dates() {
        let builder = VCmpBuilder::new()
            .calendar_id(1)
            .uid("test-uid")
            .event_type(EventType::Task)
            .summary("Unscheduled Task");

        let new_vcmp = builder.build_new().unwrap();

        match new_vcmp {
            NewVCmp::Todo(todo) => {
                assert_eq!(todo.summary, "Unscheduled Task");
                assert!(todo.starts_at.is_none());
                assert!(todo.ends_at.is_none());
            }
            NewVCmp::Event(_) => panic!("Expected Todo, got Event"),
        }
    }

    #[test]
    fn test_from_vevent_round_trip() {
        let now = Utc::now();
        let later = now + chrono::Duration::hours(1);

        let original_event = VEvent {
            id: 1,
            calendar_id: 1,
            uid: "test-uid".to_string(),
            href: Some("/test".to_string()),
            ical_data: None,
            summary: "Test Event".to_string(),
            description: Some("Description".to_string()),
            starts_at: now,
            ends_at: later,
            has_rrule: false,
            rrule_str: None,
            tag: Some("work".to_string()),
            status: EventStatus::Todo,
            event_type: EventType::Event,
            original_text: None,
            load: 5,
            urgency: 3,
            importance: 4,
            postponed: 0,
            last_modified: now.timestamp(),
            etag: Some("etag123".to_string()),
            synced_at: now.timestamp(),
        };

        let rebuilt = VCmpBuilder::from(&original_event).build().unwrap();

        match rebuilt {
            VCmp::Event(event) => {
                assert_eq!(event.id, original_event.id);
                assert_eq!(event.summary, original_event.summary);
                assert_eq!(event.description, original_event.description);
                assert_eq!(event.load, original_event.load);
            }
            VCmp::Todo(_) => panic!("Expected Event, got Todo"),
        }
    }

    #[test]
    fn test_from_vtodo() {
        let now = Utc::now();

        let original_todo = VTodo {
            id: 1,
            calendar_id: 1,
            uid: "test-uid".to_string(),
            href: Some("/test".to_string()),
            ical_data: None,
            summary: "Test Task".to_string(),
            description: Some("Description".to_string()),
            starts_at: None,
            ends_at: None,
            has_rrule: false,
            rrule_str: None,
            tag: Some("work".to_string()),
            status: EventStatus::Todo,
            event_type: EventType::Task,
            original_text: None,
            load: 5,
            urgency: 3,
            importance: 4,
            postponed: 0,
            last_modified: now.timestamp(),
            etag: Some("etag123".to_string()),
            synced_at: now.timestamp(),
            completed: None,
        };

        let rebuilt = VCmpBuilder::from(&original_todo).build().unwrap();

        match rebuilt {
            VCmp::Todo(todo) => {
                assert_eq!(todo.id, original_todo.id);
                assert_eq!(todo.summary, original_todo.summary);
                assert_eq!(todo.description, original_todo.description);
                assert_eq!(todo.load, original_todo.load);
            }
            VCmp::Event(_) => panic!("Expected Todo, got Event"),
        }
    }

    #[test]
    fn test_missing_required_field() {
        let builder = VCmpBuilder::new().uid("test-uid").summary("Test");

        let result = builder.build_new();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("calendar_id"));
    }

    #[test]
    fn test_rrule_str_sets_has_rrule() {
        let builder = VCmpBuilder::new()
            .calendar_id(1)
            .uid("test-uid")
            .summary("Recurring Event")
            .starts_at(Utc::now())
            .event_type(EventType::Event)
            .ends_at(Utc::now() + chrono::Duration::hours(1))
            .rrule_str("FREQ=DAILY");

        let new_vcmp = builder.build_new().unwrap();

        match new_vcmp {
            NewVCmp::Event(event) => {
                assert!(event.has_rrule);
                assert_eq!(event.rrule_str, Some("FREQ=DAILY".to_string()));
            }
            NewVCmp::Todo(_) => panic!("Expected Event, got Todo"),
        }
    }

    #[test]
    fn test_from_event_upsert_info() {
        use crate::calendar_items::{event_date::EventDateInfo, event_tags::EventTags};

        let date_of_input = chrono_tz::America::Buenos_Aires
            .with_ymd_and_hms(2025, 3, 6, 10, 30, 0)
            .unwrap();
        let later = date_of_input + chrono::Duration::hours(2);

        let date_info = EventDateInfo::once(date_of_input, later);

        let upsert_info = EventUpsertInfo {
            summary: "Team Meeting".to_string(),
            date_info: crate::calendar_items::event_date::EventDateOption(Some(date_info)),
            status: EventStatus::Todo,
            event_type: EventType::Event,
            postponed: 0,
            urgency: 5,
            load: 3,
            importance: 8,
            tag: EventTags(Some("work".to_string())),
        };

        let builder = VCmpBuilder::from(&upsert_info)
            .calendar_id(1)
            .uid("test-uid");

        let new_vcmp = builder.build_new().unwrap();

        match new_vcmp {
            NewVCmp::Event(event) => {
                assert_eq!(event.summary, "Team Meeting");
                assert_eq!(event.status, EventStatus::Todo);
                assert_eq!(event.event_type, EventType::Event);
                assert_eq!(event.urgency, 5);
                assert_eq!(event.load, 3);
                assert_eq!(event.importance, 8);
                assert_eq!(event.tag, Some("work".to_string()));
                assert_eq!(event.starts_at, date_of_input.to_utc());
                assert_eq!(event.ends_at, later.to_utc());
            }
            NewVCmp::Todo(_) => panic!("Expected Event, got Todo"),
        }
    }

    #[test]
    fn test_from_event_upsert_info_without_dates() {
        use crate::calendar_items::event_tags::EventTags;

        let upsert_info = EventUpsertInfo {
            summary: "Buy groceries".to_string(),
            date_info: EventDateOption::<chrono_tz::Tz>(None),
            status: EventStatus::Todo,
            event_type: EventType::Task,
            postponed: 0,
            urgency: 2,
            load: 1,
            importance: 3,
            tag: EventTags(Some("personal".to_string())),
        };

        let builder = VCmpBuilder::from(&upsert_info)
            .calendar_id(1)
            .uid("test-uid-2");

        let new_vcmp = builder.build_new().unwrap();

        match new_vcmp {
            NewVCmp::Todo(todo) => {
                assert_eq!(todo.summary, "Buy groceries");
                assert_eq!(todo.status, EventStatus::Todo);
                assert_eq!(todo.event_type, EventType::Task);
                assert_eq!(todo.urgency, 2);
                assert_eq!(todo.load, 1);
                assert_eq!(todo.importance, 3);
                assert_eq!(todo.tag, Some("personal".to_string()));
                assert!(todo.starts_at.is_none());
                assert!(todo.ends_at.is_none());
            }
            NewVCmp::Event(_) => panic!("Expected Todo, got Event"),
        }
    }
}
