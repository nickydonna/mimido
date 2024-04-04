import ICAL from 'ical.js'

/**
 * Tuple with timezone info (tzid, tzinfo)
 * @typedef {[string, string]} TzTuple
 */

/** @type {TzTuple} */
const laTz = [
  "America/Los_Angeles",
  `
BEGIN:VCALENDAR
PRODID:-//tzurl.org//NONSGML Olson 2012h//EN
VERSION:2.0
BEGIN:VTIMEZONE
TZID:America/Los_Angeles
X-LIC-LOCATION:America/Los_Angeles
BEGIN:DAYLIGHT
TZOFFSETFROM:-0800
TZOFFSETTO:-0700
TZNAME:PDT
DTSTART:19700308T020000
RRULE:FREQ=YEARLY;BYMONTH=3;BYDAY=2SU
END:DAYLIGHT
BEGIN:STANDARD
TZOFFSETFROM:-0700
TZOFFSETTO:-0800
TZNAME:PST
DTSTART:19701101T020000
RRULE:FREQ=YEARLY;BYMONTH=11;BYDAY=1SU
END:STANDARD
END:VTIMEZONE
END:VCALENDAR
  `
]

/** @type {TzTuple} */
const baTz = [
  "America/Argentina/Buenos_Aires",
  `
BEGIN:VCALENDAR
PRODID:-//tzurl.org//NONSGML Olson 2012h//EN
VERSION:2.0
BEGIN:VTIMEZONE
TZID:America/Argentina/Buenos_Aires
BEGIN:STANDARD
DTSTART:19700101T000000
TZOFFSETFROM:-0300
TZOFFSETTO:-0300
END:STANDARD
END:VTIMEZONE
END:VCALENDAR
  `
]

/** @param {TzTuple} tzInfo */
function registerTz(tzInfo) {
  if (ICAL.TimezoneService.has(tzInfo[0])) return;
  const parsed = ICAL.parse(tzInfo[1]);
  const comp = new ICAL.Component(parsed);
  let vtimezone = comp.getFirstSubcomponent('vtimezone');
  if (!vtimezone) throw new Error('Could not find vtimezone');
  ICAL.TimezoneService.register(vtimezone);
}


const tzs = [baTz, laTz];

export default function registerAllTz() {
  tzs.forEach(tz => registerTz(tz));
};