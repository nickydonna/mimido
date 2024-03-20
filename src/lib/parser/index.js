import * as chrono from 'chrono-node';
import { isSameDay } from 'date-fns';
import { format } from 'date-fns/fp';

/** @typedef {import('$lib/server/calendar').TEventSchema} TEventSchema */

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
const tagRE = /( |^)?#(?<match>[a-z0-9]+)( |$)/g
const loadRE = /( |^)(?<match>\${1,3})( |$)/;
const urgencyRE = /( |^)(?<match>\^{1,3})( |$)/;
const pImportanceRE = /( |^)(?<match>!{1,3})( |$)/;
const nImportanceRE = /( |^)(?<match>\?{1,3})( |$)/;
const dateRE = /(^| )\((?<match>.*)\)( |$)/;

/**
 * Parses text and transforms it into {TEventSchema} 
 * @param {string} str 
 * @param {Date} ref 
 * @returns {Omit<TEventSchema, 'eventId'>}
 */
export function parseTaskText(str, ref) {
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
  
  const dateMatch = title.match(dateRE);
  if (dateMatch?.groups?.['match']) {
    title = title.replace(dateMatch[0], '')
    const parsedDate = chrono.parse(dateMatch.groups['match'], ref)?.[0];
 
    if (parsedDate) {
      date = parsedDate.start.date();
      if (parsedDate.end) {
        endDate = parsedDate.end.date();
      }
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
  }
}

/**
 * Takes an event and transforms it into an string so it can be editted
 * This way we avoid issues with relative dates in text
 * @param {TEventSchema} event
 * @return {string}
 */
export function unparseTaskText(event) {
  let text = event.title
  if (event.date) {
    text += ` (${format('MMM dd HH:mm', event.date)}`
    if (event.endDate) {
     const timeFormat = isSameDay(event.date, event.endDate)
        ? 'HH:mm'
        : 'MMM dd HH:mm'
      text = text + ' until ' + format(timeFormat, event.endDate)
    }
    text += ')'
  }
  text += ` @${ event.type } %${ event.status }`;

  event.tag.forEach(t => (text += ` #${t}`))

  if (event.importance !== 0) {
    const symbol = event.importance > 0 ? '!' : '?';
    text += ` ${repeat(symbol, event.importance)}`
  }

  if (event.load) text += ` ${repeat('$', event.load)}`
  if (event.urgency) text += ` ${repeat('$', event.urgency)}`
  
  return text;
}

/**
 * Repeats the symbol the provided amount of times in a string
 * @param {string} symbol - Usually a single char 
 * @param {number} times - can be negative, will use the absolute value
 * @returns {string}
 */
function repeat(symbol, times) {
  return [...Array(Math.abs(times))].map(() => symbol).join('');
}