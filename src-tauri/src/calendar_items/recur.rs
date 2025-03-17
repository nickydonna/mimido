use rrule::RRuleSet;

use crate::calendar_items::component_props::{get_string_property, ComponentProps};

pub fn parse_rrule(event: &icalendar::Event) -> Option<RRuleSet> {
    let rrule = get_string_property(event, ComponentProps::RRule);
    let dt_start = get_string_property(event, ComponentProps::DtStart);
    let r_date = get_string_property(event, ComponentProps::RDate);
    let ex_date = get_string_property(event, ComponentProps::Exdate);

    let (Some(rrule), Some(dt_start)) = (rrule, dt_start) else {
        return None;
    };

    let mut rule_set_string = format!(
        "DTSTART:{dt_start}\n\
        RRULE:{rrule}"
    );

    if let Some(r_date) = r_date {
        rule_set_string = format!(
            "
        {rule_set_string}\n\
        RDATE:{r_date}"
        );
    }

    if let Some(ex_date) = ex_date {
        rule_set_string = format!(
            "
        {rule_set_string}\n\
        EXDATE:{ex_date}"
        );
    }

    let rrule: Option<RRuleSet> = rule_set_string.parse().ok();
    rrule
}

#[cfg(test)]
mod tests {

    use chrono::TimeZone;
    use rrule::Tz;

    use super::*;

    #[test]
    fn gets_the_value() {
        let cal: icalendar::Calendar = "BEGIN:VCALENDAR
PRODID:-//CyrusIMAP.org/Cyrus
BEGIN:VEVENT
SUMMARY:Work
UID:vevent-59e93661-2340-4faf-a84b-f980c5e0a6dd
X-ORIGINAL-TEXT:Work @block (May 20 10:00 until 13:00 | every weekday) #c:
 teal
X-TYPE:block
X-POSTPONED:0
CATEGORIES:c\\:teal
X-TAG:c:teal
CLASS:PRIVATE
TRANSP:TRANSPARENT
SEQUENCE:0
DTSTART:20240520T130000Z
DTEND:20240520T160000Z
LAST-MODIFIED:20240617T124048Z
DTSTAMP:20240617T124048Z
CREATED:20240527T171003Z
RRULE:FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR
END:VEVENT
END:VCALENDAR
"
        .to_string()
        .parse()
        .unwrap();

        let component = cal.components.first().unwrap();
        let event = component.as_event().unwrap();

        let result = parse_rrule(event);
        assert!(result.is_some());
        let rrule_set = result.unwrap();
        assert_eq!(
            *rrule_set.get_dt_start(),
            Tz::UTC.with_ymd_and_hms(2024, 5, 20, 13, 0, 0).unwrap()
        );
    }
}
