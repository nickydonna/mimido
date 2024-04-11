import { readable } from 'svelte/store';
import { EStatus, EType } from './parser';
import memoize from 'just-memoize';
import type {
	TAllTypes,
	TBlockSchema,
	TAllTypesWithId,
	WithId,
	TTaskSchema,
	TReminderSchema,
	TEventSchema
} from '$lib/server/calendar';

const IMPORTANCE_STRINGS = ['Sub-Zero', 'Very Low', 'Low', undefined, 'Mid', 'High', 'Very High'];
const URGENCY_STRINGS = [undefined, 'Soon', 'Next Up', 'Why are you not doing it'];
const LOAD_STRINGS = [undefined, 'Mid', 'Hard', 'Fat Rolling'];

function withSufix(str: string | undefined, sufix: string | undefined): string {
	if (!str) return '';
	if (!sufix) return str;
	return `${str} ${sufix}`;
}

function getString(list: Array<string | undefined>) {
	return (n: number = 0, sufix?: string) => withSufix(list[n], sufix);
}

export const importanceToString = (importance: number = 0, sufix?: string) =>
	getString(IMPORTANCE_STRINGS)(importance + 3, sufix);
export const urgencyToString = getString(URGENCY_STRINGS);
export const loadToString = getString(LOAD_STRINGS);

export function getRanking(event: TAllTypes) {
	if (isBlock(event)) return 0;
	const { urgency, load, importance } = event;
	return urgency + load + importance;
}

export const getEventColor = memoize(function (event: TAllTypes) {
	const EDefaultEventColor: Record<EType, string> = {
		[EType.BLOCK]: 'indigo',
		[EType.EVENT]: 'green',
		[EType.TASK]: 'pink',
		[EType.REMINDER]: 'blue'
	};
	const { tags, type } = event;

	const lcTags = tags.map((t) => t.toLowerCase());
	const colorTag = lcTags.find((t) => t.startsWith('c:'));
	return colorTag?.replace('c:', '') ?? EDefaultEventColor[type as EType];
});

export const getEventCardClass = memoize(function (event: TAllTypes) {
	const { tags, type } = event;
	const isBlock = type === EType.BLOCK;
	const opacity = !isBlock ? 'bg-opacity-45' : '';
	const lcTags = tags.map((t) => t.toLowerCase());
	const bgTag = lcTags.find((t) => t.startsWith('bg:'));
	if (bgTag) {
		return `card__bg-${bgTag.replace('bg:', '')}`;
	}

	const color = getEventColor(event);
	return isBlock
		? `${opacity} bg-polka-${color}-800 border-${color}-900`
		: `${opacity} bg-${color}-600 border-${color}-900`;
});

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

export function isDefined<T>(obj: T | undefined): obj is NonNullable<T> {
	return typeof obj !== 'undefined' && obj !== null;
}

export function isBlock(obj: TAllTypesWithId | undefined): obj is NonNullable<WithId<TBlockSchema>>;
export function isBlock(obj: TAllTypes | undefined): obj is NonNullable<TBlockSchema>;
export function isBlock(obj: TAllTypesWithId | TAllTypes | undefined): boolean {
	return isDefined(obj) && obj.type === EType.BLOCK;
}

export function isTask(obj: TAllTypesWithId | undefined): obj is NonNullable<WithId<TTaskSchema>>;
export function isTask(obj: TAllTypes | undefined): obj is NonNullable<TTaskSchema>;
export function isTask(obj: TAllTypesWithId | TAllTypes | undefined): boolean {
	return isDefined(obj) && obj.type === EType.TASK;
}

export function isReminder(
	obj: TAllTypesWithId | undefined
): obj is NonNullable<WithId<TReminderSchema>>;
export function isReminder(obj: TAllTypes | undefined): obj is NonNullable<TReminderSchema>;
export function isReminder(obj: TAllTypesWithId | TAllTypes | undefined): boolean {
	return isDefined(obj) && obj.type === EType.REMINDER;
}

export function isEvent(obj: TAllTypesWithId | undefined): obj is NonNullable<WithId<TEventSchema>>;
export function isEvent(obj: TAllTypes | undefined): obj is NonNullable<TEventSchema>;
export function isEvent(obj: TAllTypesWithId | TAllTypes | undefined): boolean {
	return isDefined(obj) && obj.type === EType.EVENT;
}

export function isDone(
	obj: TAllTypesWithId | undefined
): obj is NonNullable<WithId<TReminderSchema | TTaskSchema>>;
export function isDone(
	obj: TAllTypes | undefined
): obj is NonNullable<TTaskSchema | TReminderSchema>;
export function isDone(event: TAllTypesWithId | TAllTypes | undefined): boolean {
	return (isTask(event) || isReminder(event)) && event.status === EStatus.DONE;
}
