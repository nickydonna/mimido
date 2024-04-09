import yup from 'yup';

/**
 * @template {import("yup").ISchema<any, any>} T
 * @typedef {import('yup').InferType<T>} InferType
 */

/**
 * @typedef {Object} TEventMeta
 * @prop {'vtodo' | 'vevent'} icalType
 * @prop {Date} [recurrenceId]
 */
export const alarmSchema = yup.object({
  related: yup.string().matches(/START/).required(),
  isNegative: yup.boolean().required(),
  duration: yup.object({
    years: yup.number(),
    months: yup.number(),
    weeks: yup.number(),
    days: yup.number(),
    hours: yup.number(),
    minutes: yup.number(),
    seconds: yup.number(),
  }).required()
}).required();

/** @typedef {InferType<typeof alarmSchema>} TAlarm */