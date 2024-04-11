<script lang="ts">
	import {
		importanceToString,
		isDone,
		isReminder,
		isTask,
		loadToString,
		urgencyToString
	} from '$lib/util.js';
	import { differenceInMinutes } from 'date-fns';
	import { ArrowsRepeatOutline, BellActiveAltOutline } from 'flowbite-svelte-icons';
	import type { TAllTypesWithId } from '$lib/server/calendar';

	export let event: TAllTypesWithId;
	const sizeClass =
		differenceInMinutes(event.date as Date, event.endDate as Date) < 16
			? 'text-xs'
			: 'text-[0.6rem]';

	let importance: number | undefined, load: number | undefined, urgency: number | undefined;
	if (isTask(event) || isReminder(event)) {
		importance = event.importance;
		load = event.load;
		urgency = event.urgency;
	}
</script>

<div
	class:line-through={isDone(event)}
	class:text-gray-400={isDone(event)}
	class:text-gray-300={!isDone(event)}
	class={sizeClass}
>
	<p>
		<span class:line-through={isDone(event)} class:text-gray-400={isDone(event)}>
			{event.title}
		</span>
		{#if event.recur}
			<ArrowsRepeatOutline class="inline-block" />
		{/if}
		{#if event.alarms.length > 0}
			<BellActiveAltOutline class="inline-block" />
		{/if}
	</p>
	{#if (isTask(event) || isReminder(event)) && !isDone(event)}
		{importanceToString(importance, '|')}
		{urgencyToString(urgency, '|')}
		{loadToString(load)}
	{/if}
</div>
