use chrono::{DateTime, FixedOffset};

use crate::{db_conn::DbConn, util::Href};

pub(crate) trait ById: Sized {
    async fn by_id(conn: DbConn, id: i32) -> anyhow::Result<Option<Self>>;
}

pub(crate) trait ByHref: Sized {
    async fn by_href(conn: DbConn, href: &Href) -> anyhow::Result<Option<Self>>;
}

pub(crate) trait ListAll: Sized {
    async fn list_all(conn: DbConn) -> anyhow::Result<Vec<Self>>;
}

pub(crate) trait DeleteById: Sized {
    async fn delete_by_id(conn: DbConn, id: i32) -> anyhow::Result<bool>;
}

pub(crate) trait DeleteAllByCalendar: Sized {
    async fn delete_all_by_calendar(conn: DbConn, calendar_id: i32) -> anyhow::Result<()>;
}

pub(crate) trait ListForDayOrRecurring: Sized {
    async fn list_for_day_or_recurring(
        conn: DbConn,
        date: DateTime<FixedOffset>,
    ) -> anyhow::Result<Vec<Self>>;
}
