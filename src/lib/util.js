import { EType } from "./parser";
import memoize from 'just-memoize';

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
 * @param {import("./parser").ParsedEventSchema} event 
 */
export function getRanking(event) {
  const { urgency, load, importance } = event;
  return urgency + load + importance;
}

/** @enum {string} */
const EDefaultEventColor = {
  [EType.BLOCK]: 'indigo',
  [EType.EVENT]: 'green',
  [EType.TASK]: 'pink',
  [EType.REMINDER]: 'blue',
}

export const isBlock = memoize(
  /**
   * @param {import("./parser").ParsedEventSchema} event 
   */  function (event) {
    return event.type === EType.BLOCK;
  }
)

export const getEventColor = memoize(
  /**
   * @param {import("./parser").ParsedEventSchema} event 
   */
  function (event) {
    const { tag, type } = event;

    const lcTags = tag.map(t => t.toLowerCase())
    const colorTag = lcTags.find(t => t.startsWith('c:'));
    return colorTag?.replace('c:', '') ?? EDefaultEventColor[type];
  }
)

export const getEventCardClass = memoize(
  /**
   * @param {import("./parser").ParsedEventSchema} event 
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