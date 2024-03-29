import { readable } from "svelte/store";
import { EType } from "./parser";
import memoize from 'just-memoize';

/** @typedef {import('$lib/server/calendar').TAllTypes} TAllTypes */
/** @typedef {import('$lib/server/calendar').TAllTypesWithId} TAllTypesWithId */

const IMPORTANCE_STRINGS = ['Sub-Zero', 'Very Low', 'Low', undefined, 'Mid', 'High', 'Very High']
const URGENCY_STRINGS = [undefined, 'Soon', 'Next Up', 'Why are you not doing it']
const LOAD_STRINGS = [undefined, 'Mid', 'Hard', 'Fat Rolling']

/**
 * @param {string | undefined} str
 * @param {string | undefined} sufix
 * @return {string}
 */
function withSufix(str, sufix) {
  if (!str) return '';
  if (!sufix) return str;
  return `${str} ${sufix}`;
}

/**
 * 
 * @param {Array<string | undefined>} list 
 * @returns {(value?: number, sufix?: string) => string}
 */
function getString(list) {
  return (n = 0, sufix) =>  withSufix(list[n], sufix)
}

/**
 * @param {number} [importance]
 * @param {string | undefined} [sufix]
 * @return {string}
 */
export const importanceToString = (importance = 0, sufix) =>
  getString(IMPORTANCE_STRINGS)(importance + 3, sufix)
export const urgencyToString = getString(URGENCY_STRINGS)
export const loadToString = getString(LOAD_STRINGS)

/**
 * TODO Ask tessy
 * @param {TAllTypes} event 
 */
export function getRanking(event) {
  if (isBlock(event)) return 0;
  const { urgency, load, importance } = event;
  return urgency + load + importance;
}

export const getEventColor = memoize(
  /**
   * @param {TAllTypes} event 
   */
  function (event) {
    /** @enum {string} */
    const EDefaultEventColor = {
      [EType.BLOCK]: 'indigo',
      [EType.EVENT]: 'green',
      [EType.TASK]: 'pink',
      [EType.REMINDER]: 'blue',
    }
    const { tag, type } = event;

    const lcTags = tag.map(t => t.toLowerCase())
    const colorTag = lcTags.find(t => t.startsWith('c:'));
    return colorTag?.replace('c:', '') ?? EDefaultEventColor[type];
  }
)

export const getEventCardClass = memoize(
  /**
   * @param {TAllTypes} event 
   */
  function (event) {
    const { tag, type } = event;
    const isBlock = type === EType.BLOCK;
    const opacity = !isBlock ? 'bg-opacity-45' : '';
    const lcTags = tag.map(t => t.toLowerCase())
    const bgTag = lcTags.find(t => t.startsWith('bg:'))
    if (bgTag) {
      return `card__bg-${bgTag.replace('bg:', '')}`
    }

    const color = getEventColor(event)
    return isBlock
      ? `${opacity} bg-polka-${color}-600 border-${color}-600`
      : `${opacity} bg-${color}-400 border-${color}-600`;
  }
)


/**
 * Svelte Store that ticks every minute
 */
export const timeStore = readable(new Date(), (set) => {
	set(new Date());

	const interval = setInterval(() => {
		set(new Date());
	}, 10000);

	return () => clearInterval(interval);
});

/**
 * @template T
 * @param {T | undefined | null} obj 
 * @returns {obj is NonNullable<T>}
 */
export function isDefined(obj) {
  return typeof obj !== 'undefined' && obj !== null;
}

	/** 
	 * @template {import('$lib/server/calendar').TAllTypes} T
	 * @typedef {import('$lib/server/calendar').WithId<T>} WithId
	 */
/** @typedef {import('$lib/server/calendar').TBlockSchema} TBlockSchema */
/** @typedef {import('$lib/server/calendar').TTaskSchema} TTaskSchema */
/** @typedef {import('$lib/server/calendar').TEventSchema} TEventSchema */
/** @typedef {import('$lib/server/calendar').TReminderSchema} TReminderSchema */


/**
 * @overload
 * @param {TAllTypesWithId | undefined} obj 
 * @returns {obj is NonNullable<WithId<TBlockSchema>>}
 */
/**
 * @overload
 * @param {TAllTypes | undefined} obj 
 * @returns {obj is NonNullable<TBlockSchema>}
 */
/**
 * @param {TAllTypes | TAllTypesWithId  | undefined} obj 
 * @returns {boolean} 
 */
export function isBlock(obj) {
  return isDefined(obj) && obj.type === EType.BLOCK
}

/**
 * @overload
 * @param {TAllTypesWithId | undefined} obj 
 * @returns {obj is NonNullable<WithId<TTaskSchema>>} 
 */
/**
 * @overload
 * @param {TAllTypes | undefined} obj 
 * @returns {obj is NonNullable<TTaskSchema>} 
 */
/**
 * @param {TAllTypesWithId | TAllTypes | undefined} obj 
 * @returns {boolean}
 */
export function isTask(obj) {
  return isDefined(obj) && obj.type === EType.TASK
}

/**
 * @overload
 * @param {TAllTypesWithId | undefined} obj 
 * @returns {obj is NonNullable<WithId<TReminderSchema>>}
 */
/**
 * @overload
 * @param {TAllTypes | undefined} obj 
 * @returns {obj is NonNullable<TReminderSchema>}
 */
/**
 * @param {TAllTypes | TAllTypesWithId | undefined} obj 
 * @returns {boolean}
 */
export function isReminder(obj) {
  return isDefined(obj) && obj.type === EType.REMINDER
}

/**
 * @overload
 * @param {TAllTypesWithId | TAllTypes | undefined} obj 
 * @returns {obj is NonNullable<WithId<TEventSchema>>} 
 */
/**
 * @overload
 * @param {TAllTypes | undefined} obj 
 * @returns {obj is NonNullable<TEventSchema>} 
 */
/**
 * @param {TAllTypes | TAllTypesWithId | undefined} obj 
 & @returns {boolean}
 */
export function isEvent(obj) {
  return isDefined(obj) && obj.type === EType.EVENT
}