import ICAL from 'ical.js'

/** @type {Record<string, string>} */
const tzFiles = import.meta.glob('@zoneinfo/**/*.ics', {
  eager: true,
  query: '?raw',
  import: 'default',
}); // Import all

/** @param {string} tzInfo */
function registerTz(tzInfo) {
  const parsed = ICAL.parse(tzInfo);
  const comp = new ICAL.Component(parsed);
  let vtimezone = comp.getFirstSubcomponent('vtimezone');
  if (!vtimezone) throw new Error('Could not find vtimezone');
  ICAL.TimezoneService.register(vtimezone);
}

/** @type {Promise<void[]> | undefined} */
let loaded;

export default function registerAllTz() {
  if (loaded) return loaded;
  loaded = Promise.resolve(Object.values(tzFiles).map(registerTz))
};
