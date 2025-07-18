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

	let classes = $derived([
		"event-card",
		`event-card-${event.event_type.toLowerCase()}`,
		differenceInMinutes(event.starts_at, event.ends_at as Date) < 16
			? "text-xs"
			: "text-[0.6rem]",
	]);

	let isDone = event.status === "Done";
	let isTask = event.event_type === "Task";
	let isReminder = event.event_type === "Reminder";

	let [importance, load, urgency] = $derived.by(() =>
		isTask || isReminder ? [event.importance, event.load, event.urgency] : [],
	);
</script>

<div class:text-gray-400={isDone} class:text-white={!isDone} class={classes}>
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

<style lang="postcss">
	@reference "../../../app.css";

	.event-card {
		@apply m-1 p-1 rounded-lg border shadow-2xl;
		backdrop-filter: blur(1.5px);
	}

	.event-card-event {
		@apply text-white;
		@apply glassy-shadow bg-emerald-800/50 border-green-900;
		grid-column: event;
	}

	.event-card-task {
		@apply glassy-shadow bg-pink-600 border-pink-900;
		grid-column: task;
	}
	.event-card-reminder {
		@apply glass bg-blue-600 border-blue-900;
		grid-column: reminder;
	}
</style>
