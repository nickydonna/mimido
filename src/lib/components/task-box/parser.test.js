import { describe, expect, it } from "vitest"
import { parseTaskText } from './parser';

import { startOfDay, addWeeks, subWeeks, setSeconds, setHours, setMinutes, startOfHour } from "date-fns/fp";
import { startOfWeek, startOfTomorrow  } from "date-fns";

/** @type {Array<[string, Date,object]>} */
const testCases = [
  [
    "Meeting tomorrow 12:30-14:30 #work @event",
    new Date(),
    {
      title: 'Meeting',
      type: 'event',
      date: setHours(12, setMinutes(30, startOfTomorrow())),
      endDate: setHours(14, setMinutes(30, startOfTomorrow())),
      hasStartTime: true,
      hasEndTime: true,
      tag: ['work'],
      load: 0,
      status: 'back',
      urgency: 0,
      importance: 0,
    },
  ], [
    "A lot of things next week at 9 #personal @block",
    new Date(),
    {
      title: 'A lot of things',
      type: 'block',
      date: startOfHour(setHours(9, (addWeeks(1, new Date())))),
      endDate: undefined,
      hasStartTime: true,
      hasEndTime: false,
      tag: ['personal'],
      load: 0,
      status: 'back',
      urgency: 0,
      importance: 0,
    }
  ], [
    "next monday at 10 to 12:30 #mimi @reminder Not a lot of things ??",
    subWeeks(1, new Date()),
    {
      title: 'Not a lot of things',
      type: 'reminder',
      date: startOfHour(setHours(10, startOfWeek(new Date(), { weekStartsOn: 1 }))),
      endDate: setSeconds(0, setMinutes(30, setHours(12, startOfDay(startOfWeek(new Date(), { weekStartsOn: 1 }))))),
      hasStartTime: true,
      hasEndTime: true,
      tag: ['mimi'],
      load: 0,
      status: 'back',
      urgency: 0,
      importance: -2,
    }
  ], [
    "next monday #mimi #asdf @reminder Not a lot of things2 ^^",
    subWeeks(1, new Date()),
    {
      title: 'Not a lot of things2',
      type: 'reminder',
      // When no time, chrone set the middle of the day as date
      date: setHours(12, startOfWeek(new Date(), { weekStartsOn: 1 })),
      endDate: undefined, 
      hasStartTime: false,
      hasEndTime: false,
      tag: ['mimi', 'asdf'],
      load: 0,
      status: 'back',
      urgency: 2,
      importance: 0,
    }
  ], [
    "next monday %done #mimi2 @reminder aaa $$$",
    subWeeks(1, new Date()),
    {
      title: 'aaa',
      type: 'reminder',
      // When no time, chrone set the middle of the day as date
      date: setHours(12, startOfWeek(new Date(), { weekStartsOn: 1 })),
      endDate: undefined, 
      hasStartTime: false,
      hasEndTime: false,
      tag: ['mimi2'],
      load: 3,
      status: 'done',
      urgency: 0,
      importance: 0,
    }
  ]
]

describe('Task Test Parser', () => {
  testCases.forEach(([txt, date, result]) => {
    it(`Parses "${txt}"`, () => {
      expect(parseTaskText(txt, date)).toEqual({ ...result, originalText: txt })
    })
  })
})