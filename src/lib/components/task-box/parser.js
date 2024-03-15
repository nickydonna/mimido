import * as chrono from 'chrono-node';
/** @typedef {import('$lib/server/schemas/event').TEventSchema} TEventSchema */

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
  let hasStartTime = false;
  let hasEndTime = false;
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
  
  const parsedDate = chrono.parse(str, ref)?.[0];
  if (parsedDate) {
    title = title.replace(parsedDate.text, ' ');
    date = parsedDate.start.date();
    hasStartTime = parsedDate.start.isCertain('hour')
    if (parsedDate.end) {
      endDate = parsedDate.end.date();
      hasEndTime = parsedDate.end.isCertain('hour');
    }
  }

  return {
    originalText: str,
    title: title.trim(),
    type,
    date,
    endDate,
    hasStartTime,
    hasEndTime,
    tag,
    load,
    status,
    importance,
    urgency,
  }
}

