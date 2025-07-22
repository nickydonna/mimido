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

<div class="size-full p-2">
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
</div>

<style lang="postcss">
	@reference "../../../app.css";

	.event-card {
		@apply p-1 rounded-lg glassy-shadow size-full box-border;
		backdrop-filter: blur(10px);
	}

	.event-card-event {
		@apply bg-teal-600/30;
	}

	.event-card-task {
		@apply bg-task-600/30;
	}
	.event-card-reminder {
		@apply bg-reminder-800/50;
	}
</style>
