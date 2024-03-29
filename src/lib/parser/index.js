import { isBlock, isReminder, isTask } from '$lib/util';
import { parseRRule, tryParseTextForRRule } from '$lib/utils/rrule';
import * as chrono from 'chrono-node';
import { isSameDay } from 'date-fns';
import { format } from 'date-fns/fp';

/** @typedef {import('$lib/server/calendar').TAllTypesWithId} TAllTypesWithId */
/** @typedef {import('$lib/server/calendar').TAllTypes} TAllTypes */
/** @typedef {import('$lib/server/calendar').TAlarm} TAlarm */

/** @enum {string} */
export const EType = {
	EVENT: 'event',
	BLOCK: 'block',
	REMINDER: 'reminder',
	TASK: 'task',
}

/** @enum {string} */
export const EStatus = {
  BACK: 'back', // backlog
  TODO: 'todo',
  DOING: 'doing',
  DONE: 'done',
};

// eslint-disable-next-line no-useless-escape
const typeRE = new RegExp("@(?<match>" + Object.values(EType).join('|') + ")( |$)");
// eslint-disable-next-line no-useless-escape
const statusRE = new RegExp("%(?<match>" + Object.values(EStatus).join('|') + ")( |$)");

const tagRE = /( |^)?#(?<match>(:?bg:|c:)?:?[a-z0-9]+)( |$)/g
const alarmRE = /( |^)?\*(?<match>[A-Z0-9]+)( |$)/g
const loadRE = /( |^)(?<match>\${1,3})( |$)/;
const urgencyRE = /( |^)(?<match>\^{1,3})( |$)/;
const pImportanceRE = /( |^)(?<match>!{1,3})( |$)/;
const nImportanceRE = /( |^)(?<match>\?{1,3})( |$)/;
const dateRE = /(^| )\((?<match>.*)\)( |$)/;

/**
 * Parses text and transforms it into {@link TBaseSchema} 
 * @param {string} str 
 * @param {Date} [ref] 
 * @returns {TAllTypes}
 */
export function parseTaskText(str, ref = new Date()) {
  let title = str + '';
  let type = 'task';
  let date;
  let endDate;
  /** @type {string[]} */
  let tag = [];
  let status = EStatus.BACK;
  let importance = 0;
  let load = 0;
  let urgency = 0;
  /** @type {Array<import('$lib/server/calendar').TAlarm>} */
  let alarms = [];
  /** @type {string | undefined} Formatted in iCalendar RFC */
  let recur;

  const tagMatch = title.match(tagRE);
  if (tagMatch) {
    tagMatch.forEach(m => {
      title = title.replace(m, ' ');
    })
    tag = tagMatch.map(t => t.replace('#', '').trim());
  }

  const typeMatch = title.match(typeRE);
  if (typeMatch?.groups?.['match']) {
    title = title.replace(typeMatch[0], ' ');
    type = /** @type {EType} */ typeMatch.groups['match'];
  }

  const statusMatch = title.match(statusRE);
  if (statusMatch?.groups?.['match']) {
    title = title.replace(statusMatch[0], ' ');
    status = /** @type {EStatus} */ statusMatch.groups['match'];
  }

  const loadMatch = title.match(loadRE);
  if (loadMatch?.groups?.['match']) {
    title = title.replace(loadMatch[0], ' ');
    const loadStr = loadMatch.groups['match'];
    load = loadStr.length;
  }

  const nImportanceMatch = title.match(nImportanceRE);
  const pImportanceMatch = title.match(pImportanceRE);

  if (nImportanceMatch?.groups?.['match']) {
    title = title.replace(nImportanceMatch[0], ' ');
    const nImportanceStr = nImportanceMatch.groups['match'];
    importance = -nImportanceStr.length;
  }
  
  if (pImportanceMatch?.groups?.['match']) {
    title = title.replace(pImportanceMatch[0], ' ');
    const nImportanceStr = pImportanceMatch.groups['match'];
    importance = nImportanceStr.length;
  }

  const urgencyMatch = title.match(urgencyRE);
  if (urgencyMatch?.groups?.['match']) {
    title = title.replace(urgencyMatch[0], ' ');
    const urgencyStr = urgencyMatch.groups['match'];
    urgency = urgencyStr.length;
  }

  const alarmMatch = title.match(alarmRE);
  if (alarmMatch) {
    alarmMatch.forEach(m => {
      title = title.replace(m, ' ');
    })
    alarms = alarmMatch.map(t => {
      try {
        // Force duration to be negative so alarm is before
        return alarmFromString(`-${t.trim()}`);
      } catch (e) {
        console.log(e);
        return undefined;
      }
    }).filter(
      /**
       * @param {TAlarm | undefined} a
       * @returns {a is TAlarm}
       */
      a => !!a
    );
  }
  
  const dateMatch = title.match(dateRE);
  if (dateMatch?.groups?.['match']) {
    title = title.replace(dateMatch[0], '')
    const [datePart, recurPart] = dateMatch.groups['match'].split('|')
    const parsedDate = chrono.parse(datePart, ref)?.[0];
 
    if (parsedDate) {
      date = parsedDate.start.date();
      if (parsedDate.end) {
        endDate = parsedDate.end.date();
      }
    }
    if (recurPart) {
      recur = tryParseTextForRRule(recurPart)?.toString(); 
    }
  }

  return {
    originalText: str,
    title: title.trim(),
    type,
    date,
    endDate,
    tag,
    load,
    status,
    importance,
    urgency,
    recur,
    alarms,
  }
}

/**
 * Takes an event and transforms it into an string so it can be editted
 * This way we avoid issues with relative dates in text
 * @param {TAllTypesWithId} event
 * @return {string}
 */
export function unparseTaskText(event) {
  const {
    title,
    date,
    endDate,
    recur,
    type,
    tag,
    alarms,
  } = event;

  let text = title

  if (isReminder(event) || isTask(event)) {
    text += ` %${event.status}`;
  }

  if (date) {
    text += ` (${format('MMM dd HH:mm', date)}`
    if (endDate) {
     const timeFormat = isSameDay(date, endDate)
        ? 'HH:mm'
        : 'MMM dd HH:mm'
      text = text + ' until ' + format(timeFormat, endDate)
    }
    if (recur) {
      text += ' | ' + parseRRule(recur).toText()
    }
    text += ')'
  }

  if (alarms.length > 0) {
    alarms.forEach(a => {
      const { weeks, days, hours, minutes } = a.duration;
      let durText = ' *P';
      if (weeks) durText += `${weeks}W`;
      if (days) durText += `${days}D`;
      if (hours || minutes) {
        durText += 'T';
        if (hours) durText += `${hours}H`;
        if (minutes) durText += `${minutes}M`;
      }
      text += durText
    })
  }
  text += ` @${ type } %${ status }`;

  tag.forEach(t => (text += ` #${t}`))

  if (isBlock(event)) return text;

  const { importance, urgency, load } = event;

  if (importance !== 0) {
    const symbol = importance > 0 ? '!' : '?';
    text += ` ${symbol.repeat(importance)}`
  }

  if (load > 0) text += ` ${'$'.repeat(load)}`
  if (urgency > 0) text += ` ${'^'.repeat(urgency)}`
  
  return text;
}

const DURATION_LETTERS = /([PDWHMTS]{1,1})/;

/**
 * Copied from https://kewisch.github.io/ical.js/api/duration.js.html#line46 
 * @param {string} aStr 
 * @returns {import('$lib/server/calendar').TAlarm}
 */
function alarmFromString(aStr) {
  let pos = 0;
  let dict = Object.create(null);
  let chunks = 0;

  while ((pos = aStr.search(DURATION_LETTERS)) !== -1) {
    let type = aStr[pos];
    let numeric = aStr.slice(0, Math.max(0, pos));
    aStr = aStr.slice(pos + 1);
    chunks += parseDurationChunk(type, numeric, dict);
  }
 
  if (chunks < 2) {
    // There must be at least a chunk with "P" and some unit chunk
     throw new Error(
      'invalid duration value: Not enough duration components in "' + aStr + '"'
    );
  }
  const { isNegative, ...duration } = dict;
  return {
    isNegative,
    duration,
    related: 'START',
  }
}

/**
 * Internal helper function to handle a chunk of a duration.
 *
 * @private
 * @param {string} letter type of duration chunk
 * @param {string} number numeric value or -/+
 * @param {Partial<import('date-fns').Duration> & { isNegative: boolean }} object target to assign values to
 */
function parseDurationChunk(letter, number, object) {
  /** @type {keyof import('date-fns').Duration | undefined} */
  let type;
  switch (letter) {
    case 'P':
      if (number && number === '-') {
        object.isNegative = true;
      } else {
        object.isNegative = false;
      }
      // period
      break;
    case 'D':
      type = 'days';
      break;
    case 'W':
      type = 'weeks';
      break;
    case 'H':
      type = 'hours';
      break;
    case 'M':
      type = 'minutes';
      break;
    case 'S':
      type = 'seconds';
      break;
    default:
      // Not a valid chunk
      return 0;
  }

  if (type) {
    let num = parseInt(number, 10);
    if (!number && num !== 0) {
      throw new Error(
        'invalid duration value: Missing number before "' + letter + '"'
      );
    }
    if (Number.isNaN(num)) {
      throw new Error(
        'invalid duration value: Invalid number "' + number + '" before "' + letter + '"'
      );
    }

    object[type] = num;

  }
  return 1;
}