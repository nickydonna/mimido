import ICAL from 'ical.js'
import fs from 'node:fs';
import path from 'path';


/** @param {string} tzInfo */
function registerTz(tzInfo) {
  const parsed = ICAL.parse(tzInfo);
  const comp = new ICAL.Component(parsed);
  let vtimezone = comp.getFirstSubcomponent('vtimezone');
  if (!vtimezone) throw new Error('Could not find vtimezone');
  ICAL.TimezoneService.register(vtimezone);
}

const directoryPath = path.join('src/lib/server/calendar/zoneinfo');

/**
 * @param {string} dir
 * @returns {string[]}
 */
function throughDirectory(dir) {
  const objs = fs.readdirSync(dir)
  console.log(objs)
  
  return objs.map(file => {
    const absolute = path.join(dir, file);
    // console.log(absolute, dir, file)
    if (fs.statSync(absolute).isDirectory()) return throughDirectory(absolute);
    else return [absolute];
  }).flat()
}

/** @type {Promise<void[]> | undefined} */
let loaded;

export default function registerAllTz() {
  if (loaded) return loaded;
  const tzs = throughDirectory(directoryPath).filter(f => f.endsWith('ics'))
  loaded = Promise.all(tzs.map(async tz => {
    const file = await fs.promises.readFile(tz, 'utf8');
    registerTz(file)
  }));
};
