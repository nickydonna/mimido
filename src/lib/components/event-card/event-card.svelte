<script>
	import { importanceToString, isReminder, isTask, loadToString, urgencyToString } from '$lib/util';
	import { differenceInMinutes, formatDuration } from 'date-fns';
	import { ArrowsRepeatOutline, BellActiveAltOutline } from 'flowbite-svelte-icons';

	/** @type {import("$lib/server/calendar").TAllTypesWithId} */
	export let event;
	const sizeClass =
		differenceInMinutes(/** @type {Date} */ (event.date), /** @type {Date} */ (event.endDate)) < 16
			? 'text-xs'
			: 'text-[0.6rem]';
</script>

<div class="text-gray-600 dark:text-gray-300 {sizeClass}">
	<p >
		{event.title}
		{#if event.recur}
			<ArrowsRepeatOutline class="inline-block" />
		{/if}
		{#if event.alarms.length > 0}
			<BellActiveAltOutline class="inline-block" />
		{/if}
	</p>
	{#if isTask(event) || isReminder(event)}
		{importanceToString(event.importance, '|')}
		{urgencyToString(event.urgency, '|')}
		{loadToString(event.load)}
	{/if}
</div>
