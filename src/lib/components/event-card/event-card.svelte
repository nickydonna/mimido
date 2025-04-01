<script lang="ts">
	import {
		importanceToString,
		loadToString,
		urgencyToString,
		type ParsedEvent,
	} from "$lib/util.js";
	import { differenceInMinutes } from "date-fns";
	import {
		ArrowsRepeatOutline,
		BellActiveAltOutline,
	} from "flowbite-svelte-icons";

	let { event }: { event: ParsedEvent } = $props();
	const sizeClass =
		differenceInMinutes(event.starts_at, event.ends_at as Date) < 16
			? "text-xs"
			: "text-[0.6rem]";

	let isDone = event.status === "Done";
	let isTask = event.event_type === "Task";
	let isReminder = event.event_type === "Reminder";

	let [importance, load, urgency] = $derived.by(() =>
		isTask || isReminder ? [event.importance, event.load, event.urgency] : [],
	);
</script>

<div
	class:line-through={isDone}
	class:text-gray-400={isDone}
	class:text-gray-300={!isDone}
	class={sizeClass}
>
	<p>
		<span class:line-through={isDone} class:text-gray-400={isDone}>
			{event.summary}
		</span>
		{#if event.natural_recurrence}
			<ArrowsRepeatOutline class="inline-block" />
		{/if}
		<!-- {#if event.alarms.length > 0} -->
		<!-- 	<BellActiveAltOutline class="inline-block" /> -->
		<!-- {/if} -->
	</p>
	{#if (isTask || isReminder) && !isDone}
		{importanceToString(importance, "|")}
		{urgencyToString(urgency, "|")}
		{loadToString(load)}
	{/if}
</div>
