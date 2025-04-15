use crate::{
    calendar_items::{
        component_props::GeneralComponentProps, event_status::EventStatus, event_type::EventType,
    },
    schema::*,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use libdav::FetchedResource;

use super::IcalParseableTrait;

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, serde::Serialize)]
#[diesel(table_name = todos)]
pub struct Todo {
    pub id: i32,
    pub calendar_id: i32,
    pub uid: String,
    pub href: String,
    pub ical_data: String,
    pub summary: String,
    pub description: Option<String>,
    pub completed: bool,
    pub tag: Option<String>,
    pub status: EventStatus,
    pub event_type: EventType,
    pub original_text: Option<String>,
    pub load: i32,
    pub urgency: i32,
    pub importance: i32,
    pub postponed: i32,
    pub last_modified: i64,
}

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug)]
#[diesel(table_name = todos)]
pub struct NewTodo {
    pub calendar_id: i32,
    pub href: String,
    pub uid: String,
    pub ical_data: String,
    pub summary: String,
    pub description: Option<String>,
    pub completed: bool,
    pub tag: Option<String>,
    pub status: EventStatus,
    pub event_type: EventType,
    pub original_text: Option<String>,
    pub load: i32,
    pub urgency: i32,
    pub importance: i32,
    pub postponed: i32,
    pub last_modified: i64,
}

impl IcalParseableTrait for Todo {
    fn get_ical_data(&self) -> String {
        self.ical_data.clone()
    }
    fn get_summary(&self) -> String {
        self.summary.clone()
    }

    fn get_description(&self) -> Option<String> {
        self.description.clone()
    }

    fn get_postponed(&self) -> i32 {
        self.postponed
    }

    fn get_load(&self) -> i32 {
        self.load
    }

    fn get_urgency(&self) -> i32 {
        self.urgency
    }

    fn get_importance(&self) -> i32 {
        self.importance
    }
    fn get_status(&self) -> EventStatus {
        self.status
    }
    fn get_type(&self) -> EventType {
        self.event_type
    }
}

impl IcalParseableTrait for NewTodo {
    fn get_ical_data(&self) -> String {
        self.ical_data.clone()
    }
    fn get_summary(&self) -> String {
        self.summary.clone()
    }

    fn get_description(&self) -> Option<String> {
        self.description.clone()
    }

    fn get_postponed(&self) -> i32 {
        self.postponed
    }

    fn get_load(&self) -> i32 {
        self.load
    }

    fn get_urgency(&self) -> i32 {
        self.urgency
    }

    fn get_importance(&self) -> i32 {
        self.importance
    }
    fn get_status(&self) -> EventStatus {
        self.status
    }
    fn get_type(&self) -> EventType {
        self.event_type
    }
}

pub(crate) trait TodoTrait: IcalParseableTrait {
    fn get_start(&self) -> Option<DateTime<Utc>>;
}

impl NewTodo {
    pub fn new_from_resource(
        calendar_id: i32,
        fetched_resource: &FetchedResource,
    ) -> Result<Option<NewTodo>, String> {
        let href = fetched_resource.href.clone();
        let content = fetched_resource
            .content
            .as_ref()
            .map_err(|e| e.to_string())?;
        NewTodo::new_from_ical_data(calendar_id, href, content.data.clone())
    }

    pub fn new_from_ical_data(
        calendar_id: i32,
        href: String,
        ical_data: String,
    ) -> Result<Option<Self>, String> {
        let calendar_item: icalendar::Calendar = ical_data.parse()?;
        let first_todo = calendar_item
            .components
            .into_iter()
            .filter_map(|cmp| cmp.as_todo().cloned())
            .collect::<Vec<icalendar::Todo>>();

        let Some(first_todo) = first_todo.first() else {
            return Ok(None);
        };

        let GeneralComponentProps {
            uid,
            summary,
            description,
            event_type,
            tag,
            urgency,
            status,
            original_text,
            importance,
            load,
            postponed,
            last_modified,
        } = GeneralComponentProps::try_from(first_todo)?;

        println!(
            "type: {:#?} {:#?} {:#?} {:#?} {:#?} {:#?}",
            event_type,
            format!("{:.10}", summary),
            status,
            importance,
            urgency,
            load
        );

        Ok(Some(NewTodo {
            calendar_id,
            uid: uid.to_string(),
            href,
            ical_data,
            last_modified,
            summary: summary.to_string(),
            // TODO: Use real completed and sycn with status
            completed: false,
            description,
            status,
            original_text,
            tag,
            event_type,
            importance,
            load,
            urgency,
            postponed,
        }))
    }
}
#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::*;
    #[test]
    fn test_should_parsed_todo() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("./fixtures/todo.ics");
        let ics = fs::read_to_string(d).expect("To Load file");
        let todo = NewTodo::new_from_ical_data(1, "test".to_string(), ics);

        assert!(todo.is_ok());
        let todo = todo.unwrap();
        assert!(todo.is_some());
        let todo = todo.unwrap();
        assert_eq!(todo.summary, "Yerba");
    }
}
