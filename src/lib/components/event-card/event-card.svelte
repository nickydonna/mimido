<script>
	import { importanceToString, loadToString, urgencyToString } from '$lib/util';
	import { differenceInMinutes, formatDuration } from 'date-fns';
	import { ArrowsRepeatOutline, BellActiveAltOutline } from 'flowbite-svelte-icons';
	import * as pkg from 'rrule';
	// @ts-expect-error - see https://github.com/jkbrzt/rrule/issues/548
	const { RRule } = pkg.default || pkg;

	/** @type {import("$lib/server/calendar").TBaseSchema} */
	export let event;
	const sizeClass =
		differenceInMinutes(/** @type {Date} */ (event.date), /** @type {Date} */ (event.endDate)) < 16
			? 'text-sm'
			: 'text-md';
</script>

<div class={sizeClass}>
	<p >
		{event.title}
		{#if event.recur}
			<ArrowsRepeatOutline class="inline-block" />
		{/if}
		{#if event.alarms.length > 0}
			<BellActiveAltOutline class="inline-block" />
		{/if}
	</p>
	{importanceToString(event.importance, '|')}
	{urgencyToString(event.urgency, '|')}
	{loadToString(event.load)}
</div>
