import { isBlock, isDefined, isReminder, isTask } from '$lib/util.js';
import { parseRRule, tryParseTextForRRule } from '$lib/utils/rrule';
import * as chrono from 'chrono-node';
import { isSameDay } from 'date-fns';
import { format } from 'date-fns/fp';
import type { TAllTypes } from '$lib/server/calendar';
import type { Duration } from 'date-fns';
import type { TAlarmSchema } from '$lib/server/calendar/alarmSchema';

export enum EType {
	EVENT = 'event',
	BLOCK = 'block',
	REMINDER = 'reminder',
	TASK = 'task'
}

export enum EStatus {
	BACK = 'back', // backlog
	TODO = 'todo',
	DOING = 'doing',
	DONE = 'done'
}

// eslint-disable-next-line no-useless-escape
const typeRE = new RegExp('@(?<match>' + Object.values(EType).join('|') + ')( |$)');
// eslint-disable-next-line no-useless-escape
const statusRE = new RegExp('%(?<match>' + Object.values(EStatus).join('|') + ')( |$)');

const tagRE = /( |^)?#(?<match>(:?bg:|c:)?:?[a-z0-9]+)( |$)/g;
const alarmRE = /( |^)?\*(?<match>[A-Z0-9]+)( |$)/g;
const loadRE = /( |^)(?<match>\${1,3})( |$)/;
const urgencyRE = /( |^)(?<match>\^{1,3})( |$)/;
const pImportanceRE = /( |^)(?<match>!{1,3})( |$)/;
const nImportanceRE = /( |^)(?<match>\?{1,3})( |$)/;
const dateRE = /(^| )\((?<match>.*)\)( |$)/;

export function parseTaskText(str: string, tzOffset?: number): TAllTypes {
	let title = str + '';
	let type = EType.TASK;
	let date;
	let endDate;
	let tags: string[] = [];
	let status = EStatus.BACK;
	let importance = 0;
	let load = 0;
	let urgency = 0;
	let alarms: TAlarmSchema[] = [];
	let recur: string | undefined = undefined;

	const tagMatch = title.match(tagRE);
	if (tagMatch) {
		tagMatch.forEach((m) => {
			title = title.replace(m, ' ');
		});
		tags = tagMatch.map((t) => t.replace('#', '').trim());
	}

	const typeMatch = title.match(typeRE);
	if (typeMatch?.groups?.['match']) {
		title = title.replace(typeMatch[0], ' ');
		type = typeMatch.groups['match'] as EType;
	}

	const statusMatch = title.match(statusRE);
	if (statusMatch?.groups?.['match']) {
		title = title.replace(statusMatch[0], ' ');
		status = statusMatch.groups['match'] as EStatus;
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
		alarmMatch.forEach((m) => {
			title = title.replace(m, ' ');
		});
		alarms = alarmMatch
			.map((t) => {
				try {
					// Force duration to be negative so alarm is before
					return alarmFromString(`-${t.trim()}`);
				} catch (e) {
					return undefined;
				}
			})
			.filter(isDefined);
	}

	const dateMatch = title.match(dateRE);
	if (dateMatch?.groups?.['match']) {
		title = title.replace(dateMatch[0], '');
		const [datePart, recurPart] = dateMatch.groups['match'].split('|');
		const parsedDate = chrono.parse(datePart, tzOffset ? { timezone: -tzOffset } : undefined)?.[0];

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
		tags,
		load,
		status,
		importance,
		urgency,
		recur,
		alarms,
		postponed: 0
	};
}

export function unparseTaskText(event: TAllTypes) {
	const { title, date, endDate, recur, type, tags, alarms } = event;

	let text = title;
	text += ` @${type}`;

	if (isReminder(event) || isTask(event)) {
		text += ` %${event.status}`;
	}

	if (date) {
		text += ` (${format('MMM dd HH:mm', date)}`;
		if (endDate) {
			const timeFormat = isSameDay(date, endDate) ? 'HH:mm' : 'MMM dd HH:mm';
			text = text + ' until ' + format(timeFormat, endDate);
		}
		if (recur) {
			text += ' | ' + parseRRule(recur).toText();
		}
		text += ')';
	}

	if (alarms.length > 0) {
		alarms.forEach((a) => {
			const { weeks, days, hours, minutes } = a.duration;
			let durText = ' *P';
			if (weeks) durText += `${weeks}W`;
			if (days) durText += `${days}D`;
			if (hours || minutes) {
				durText += 'T';
				if (hours) durText += `${hours}H`;
				if (minutes) durText += `${minutes}M`;
			}
			text += durText;
		});
	}

	tags.forEach((t) => (text += ` #${t}`));

	if (isBlock(event)) return text;

	const { importance, urgency, load } = event;

	if (importance !== 0) {
		const symbol = importance > 0 ? '!' : '?';
		text += ` ${symbol.repeat(Math.abs(importance))}`;
	}

	if (load > 0) text += ` ${'$'.repeat(load)}`;
	if (urgency > 0) text += ` ${'^'.repeat(urgency)}`;

	return text;
}

const DURATION_LETTERS = /([PDWHMTS])/;

/**
 * Copied from https://kewisch.github.io/ical.js/api/duration.js.html#line46
 */
export function alarmFromString(aStr: string): TAlarmSchema {
	let pos = 0;
	const dict = Object.create(null);
	let chunks = 0;

	while ((pos = aStr.search(DURATION_LETTERS)) !== -1) {
		const type = aStr[pos];
		const numeric = aStr.slice(0, Math.max(0, pos));
		aStr = aStr.slice(pos + 1);
		chunks += parseDurationChunk(type, numeric, dict);
	}

	if (chunks < 2) {
		// There must be at least a chunk with "P" and some unit chunk
		throw new Error('invalid duration value: Not enough duration components in "' + aStr + '"');
	}
	const { isNegative, ...duration } = dict;
	return {
		isNegative,
		duration,
		related: 'START'
	};
}

function parseDurationChunk(
	letter: string,
	number: string,
	object: Partial<Duration> & {
		isNegative: boolean;
	}
) {
	let type: keyof Duration | undefined;
	switch (letter) {
		case 'P':
			object.isNegative = !!(number && number === '-');
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
		const num = parseInt(number, 10);
		if (!number && num !== 0) {
			throw new Error('invalid duration value: Missing number before "' + letter + '"');
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
