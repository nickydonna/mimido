/**
 * Utility function to help with 
 * - error handling
 * - typings for rrule library
 * - weird imports
 */
import * as rrulepkg from 'rrule';
// @ts-expect-error - see https://github.com/jkbrzt/rrule/issues/548
const rrule = /** @type {import('rrule')} */ (rrulepkg.default || rrulepkg);

const {
  rrulestr,
  RRule,
} = rrule

/**
 * Parse string from RRule 
 * @throws {Error} on invalid RRule string
 * @param {string} str 
 */
export function parseRRule(str) {
  try {
    return rrulestr(str)
  } catch (e) {
    throw new Error('Bad RRule format', { cause: e })
  }
}

/**
 * Parse string from RRule, return undefined if invalid 
 * @param {string} str 
 */
export function tryParseRRule(str) {
  try {
    return parseRRule(str);
  } catch (e) {
    return undefined;
  }
}

/**
 * @param {string} str 
 * @returns {boolean}
 */
export function isValidRRule(str) {
  return !!tryParseRRule(str);
}

/**
 * @param {string} text 
 * @throws {Error} Throws when it can't parse the rule from NL
 */
export function parseTextForRRule(text) {
  let result;
  try {
    result = RRule.fromText(text);
  } catch (e) {
     throw new Error(`Could not parse ${text} as recur rule`, { cause: e }) 
  }
  if (Object.keys(result.origOptions).length === 0) {
    throw new Error(`Could not parse ${text} as recur rule`, { cause: 'Empty Result from prse' }) 
  }
  return result;
}

/**
 * @param {string} text 
 */
export function tryParseTextForRRule(text) {
  try {
    return parseTextForRRule(text);
  } catch (e) {
    return undefined;
  }
}
