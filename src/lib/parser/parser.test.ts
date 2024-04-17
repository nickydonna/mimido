import { describe, expect, it } from 'vitest';
import { EType, parseTaskText, unparseTaskText } from '.';

import {
	startOfDay,
	addWeeks,
	setHours,
	setMinutes,
	startOfHour,
	nextMonday,
	startOfMinute,
	subMinutes
} from 'date-fns/fp';
import { startOfTomorrow } from 'date-fns';
import type { TAllTypes } from '$lib/server/calendar';

const baseInfo = {
	tags: [],
	load: 0,
	urgency: 0,
	importance: 0,
	alarms: [],
	recur: undefined
};

function createDate(date: Date, hours: number, minutes = 0, offset = 0) {
	const d = subMinutes(offset, startOfMinute(setMinutes(minutes, setHours(hours, date))));
	return d;
}

// Tuple of original text, offset, expected
const testCases: Array<[string, number | undefined, Omit<TAllTypes, 'originalText'>]> = [
	[
		'Meeting (tomorrow 12:30-14:30) #work @event',
		0,
		{
			...baseInfo,
			title: 'Meeting',
			type: 'event',
			date: createDate(startOfTomorrow(), 12, 30),
			endDate: createDate(startOfTomorrow(), 14, 30),
			tags: ['work'],
			// @ts-expect-error - Just testing
			status: 'back'
		}
	],
	[
		'A lot of things (next week at 9) #personal @block',
		0,
		{
			...baseInfo,
			title: 'A lot of things',
			type: 'block',
			date: startOfHour(setHours(9, addWeeks(1, new Date()))),
			endDate: undefined,
			tags: ['personal'],
			// @ts-expect-error - Just testing
			status: 'back'
		}
	],
	[
		'(next monday at 10 to 12:30) #mimi @reminder With TZ ??',
		180,
		{
			...baseInfo,
			title: 'With TZ',
			type: 'reminder',
			date: createDate(nextMonday(new Date()), 13, 0),
			endDate: createDate(nextMonday(new Date()), 15, 30),
			tags: ['mimi'],
			// @ts-expect-error - Just testing
			status: 'back',
			importance: -2
		}
	],
	[
		'(next monday) #mimi #asdf @reminder Not a lot of things2 ^^',
		0,
		{
			...baseInfo,
			title: 'Not a lot of things2',
			type: 'reminder',
			// When no time, chrone set the middle of the day as date
			date: startOfHour(setHours(12, nextMonday(new Date()))),
			endDate: undefined,
			tags: ['mimi', 'asdf'],
			// @ts-expect-error - Just testing
			status: 'back',
			urgency: 2
		}
	],
	[
		'(next monday) %done #mimi2 @reminder aaa $$$',
		0,
		{
			...baseInfo,
			title: 'aaa',
			type: 'reminder',
			// When no time, chrone set the middle of the day as date
			date: startOfHour(setHours(12, nextMonday(new Date()))),
			endDate: undefined,
			tags: ['mimi2'],
			// @ts-expect-error - Just testing
			load: 3,
			status: 'done'
		}
	],
	[
		'work (today from 9:30 to 13:30 | every weekday) @block',
		0,
		{
			...baseInfo,
			title: 'work',
			type: EType.BLOCK,
			// @ts-expect-error - Just testing
			status: 'back',
			// When no time, chrone set the middle of the day as date
			date: startOfMinute(setMinutes(30, setHours(9, startOfDay(new Date())))),
			endDate: startOfMinute(setMinutes(30, setHours(13, startOfDay(new Date())))),
			recur: 'RRULE:FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR'
		}
	]
];

describe('Task Test Parser', () => {
	testCases.forEach(([txt, offset, result]) => {
		it(`Parses "${txt}"`, () => {
			const r = parseTaskText(txt, offset);
			expect(parseTaskText(txt, offset)).toEqual({ ...result, originalText: txt });
		});
	});
});

describe('Unparse Task', () => {
	testCases.forEach(([txt, , result]) => {
		it(`Un Parses "${txt}"`, () => {
			const task = { ...result, originalText: txt } as TAllTypes;
			const unparsed = unparseTaskText(task);
			const parse = parseTaskText(unparsed);
			expect(parse).toEqual({ ...result, originalText: parse.originalText });
		});
	});
});
