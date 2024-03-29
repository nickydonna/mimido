import { describe, expect, it } from "vitest"
import { EType, parseTaskText } from '.';

import { startOfDay, addWeeks, subWeeks, setSeconds, setHours, setMinutes, startOfHour } from "date-fns/fp";
import { startOfWeek, startOfTomorrow  } from "date-fns";

/** @typedef {import(".").ParsedEventSchema} ParsedEventSchema */

const baseInfo = {
  tag: [],
  load: 0,
  urgency: 0,
  importance: 0,
  alarm: undefined,
  recur: undefined
}

/** @type {Array<[string, Date, Omit<ParsedEventSchema, 'originalText'>]>} */
const testCases = [
  [
    "Meeting (tomorrow 12:30-14:30) #work @event",
    new Date(),
    {
      ...baseInfo,
      title: 'Meeting',
      type: 'event',
      date: setHours(12, setMinutes(30, startOfTomorrow())),
      endDate: setHours(14, setMinutes(30, startOfTomorrow())),
      tag: ['work'],
      status: 'back',
    },
  ], [
    "A lot of things (next week at 9) #personal @block",
    new Date(),
    {
      ...baseInfo,
      title: 'A lot of things',
      type: 'block',
      date: startOfHour(setHours(9, (addWeeks(1, new Date())))),
      endDate: undefined,
      tag: ['personal'],
      status: 'back',
    }
  ], [
    "(next monday at 10 to 12:30) #mimi @reminder Not a lot of things ??",
    subWeeks(1, new Date()),
    {
      ...baseInfo,
      title: 'Not a lot of things',
      type: 'reminder',
      date: startOfHour(setHours(10, startOfWeek(new Date(), { weekStartsOn: 1 }))),
      endDate: setSeconds(0, setMinutes(30, setHours(12, startOfDay(startOfWeek(new Date(), { weekStartsOn: 1 }))))),
      tag: ['mimi'],
      status: 'back',
      importance: -2,
    }
  ], [
    "(next monday) #mimi #asdf @reminder Not a lot of things2 ^^",
    subWeeks(1, new Date()),
    {
      ...baseInfo,
      title: 'Not a lot of things2',
      type: 'reminder',
      // When no time, chrone set the middle of the day as date
      date: setHours(12, startOfWeek(new Date(), { weekStartsOn: 1 })),
      endDate: undefined, 
      tag: ['mimi', 'asdf'],
      status: 'back',
      urgency: 2,
    }
  ], [
    "(next monday) %done #mimi2 @reminder aaa $$$",
    subWeeks(1, new Date()),
    {
      ...baseInfo,
      title: 'aaa',
      type: 'reminder',
      // When no time, chrone set the middle of the day as date
      date: setHours(12, startOfWeek(new Date(), { weekStartsOn: 1 })),
      endDate: undefined, 
      tag: ['mimi2'],
      load: 3,
      status: 'done',
    }
  ], [
    "work (today from 9:30 to 13:30 | every weekday) @block",
    new Date(),
    {
      ...baseInfo,
      title: 'work',
      type: EType.BLOCK,
      status: 'back',
      // When no time, chrone set the middle of the day as date
      date: setMinutes(30, setHours(9, startOfDay(new Date()))),
      endDate: setMinutes(30, setHours(13, startOfDay(new Date()))),
      recur: 'RRULE:FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR',
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