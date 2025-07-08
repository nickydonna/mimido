use crate::{
    calendar_items::{
        component_props::GeneralComponentProps, event_status::EventStatus, event_type::EventType,
    },
    impl_ical_parseable,
    schema::*,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use libdav::FetchedResource;

use super::IcalParseableTrait;

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, serde::Serialize)]
#[diesel(table_name = vtodos)]
pub struct VTodo {
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
#[diesel(table_name = vtodos)]
pub struct NewVTodo {
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

impl_ical_parseable!(VTodo);
impl_ical_parseable!(NewVTodo);

pub(crate) trait VTodoTrait: IcalParseableTrait {
    fn get_start(&self) -> Option<DateTime<Utc>>;
    fn to_input(&self, _: DateTime<chrono_tz::Tz>) -> String {
        format!(
            "{} {} {}",
            self.get_type(),
            self.get_status(),
            self.get_summary()
        )
    }
}

macro_rules! impl_todo_trait {
    ($t: ty) => {
        impl VTodoTrait for $t {
            fn get_start(&self) -> Option<DateTime<Utc>> {
                None
            }
        }
    };
}

impl_todo_trait!(VTodo);
impl_todo_trait!(NewVTodo);

impl NewVTodo {
    pub fn new_from_resource(
        calendar_id: i32,
        fetched_resource: &FetchedResource,
    ) -> Result<Option<NewVTodo>, String> {
        let href = fetched_resource.href.clone();
        let content = fetched_resource
            .content
            .as_ref()
            .map_err(|e| e.to_string())?;
        NewVTodo::new_from_ical_data(calendar_id, href, content.data.clone())
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

        Ok(Some(NewVTodo {
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
        let todo = NewVTodo::new_from_ical_data(1, "test".to_string(), ics);

        assert!(todo.is_ok());
        let todo = todo.unwrap();
        assert!(todo.is_some());
        let todo = todo.unwrap();
        assert_eq!(todo.summary, "Yerba");
    }
}
