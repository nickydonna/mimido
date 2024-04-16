import ICAL from 'ical.js';

const tzFiles: Record<string, string> = import.meta.glob('@zoneinfo/**/*.ics', {
	eager: true,
	query: '?raw',
	import: 'default'
}); // Import all

function registerTz(tzInfo: string) {
	const parsed = ICAL.parse(tzInfo.replace('/citadel.org/20240317_1/', ''));
	const comp = new ICAL.Component(parsed);
	const vtimezone = comp.getFirstSubcomponent('vtimezone');
	if (!vtimezone) throw new Error('Could not find vtimezone');
	ICAL.TimezoneService.register(vtimezone);
}

/** @type {Promise<void[]> | undefined} */
let loaded: Promise<void[]> | undefined;

export default function registerAllTz() {
	if (loaded) return loaded;
	loaded = Promise.resolve(Object.values(tzFiles).map(registerTz));
}
