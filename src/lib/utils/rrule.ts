/**
 * Utility function to help with
 * - error handling
 * - typings for rrule library
 * - weird imports
 */
import type { RRule as TRRule, rrulestr as Trrulestr } from 'rrule';
import * as rrulepkg from 'rrule';
// @ts-expect-error - see https://github.com/jkbrzt/rrule/issues/548
const rrule = rrulepkg.default || rrulepkg;

const RRule: typeof TRRule = rrule.RRule;
const rrulestr: typeof Trrulestr = rrule.rrulestr;

/**
 * Parse string from RRule
 * @throws {Error} on invalid RRule string
 */
export function parseRRule(str: string) {
	try {
		return rrulestr(str);
	} catch (e) {
		throw new Error('Bad RRule format', { cause: e });
	}
}

/**
 * Parse string from RRule, return undefined if invalid
 */
export function tryParseRRule(str: string) {
	try {
		return parseRRule(str);
	} catch (e) {
		return undefined;
	}
}

export function isValidRRule(str: string): boolean {
	return !!tryParseRRule(str);
}

/**
 * @throws {Error} Throws when it can't parse the rule from NL
 */
export function parseTextForRRule(text: string) {
	let result;
	try {
		result = RRule.fromText(text);
	} catch (e) {
		throw new Error(`Could not parse ${text} as recur rule`, { cause: e });
	}
	if (Object.keys(result.origOptions).length === 0) {
		throw new Error(`Could not parse ${text} as recur rule`, { cause: 'Empty Result from prse' });
	}
	return result;
}

export function tryParseTextForRRule(text: string) {
	try {
		return parseTextForRRule(text);
	} catch (e) {
		return undefined;
	}
}

export function rruleToText(rrule: string) {
	return tryParseRRule(rrule)?.toText();
}

export function addUntilDate(rrule: string, until: Date) {
	const parsed = parseRRule(rrule);
	const newRrule = new RRule({
		...parsed.origOptions,
		until,
	})
	return newRrule.toString()
}
