<script lang="ts">
	import {
		importanceToString,
		loadToString,
		urgencyToString,
		type ScheduledTask,
	} from "$lib/util.js";
	import { differenceInMinutes } from "date-fns";
	import { ArrowsRepeatOutline } from "flowbite-svelte-icons";
	// @ts-expect-error iconify
	import CheckTask from "~icons/solar/bill-check-linear";
	// @ts-expect-error iconify
	import UnCheckTask from "~icons/solar/bill-cross-linear";
	import { commands } from "../../../bindings";
	import { invalidateAll } from "$app/navigation";
	import {
		EventUpsert,
		eventUpserter,
	} from "../../../stores/eventUpserter.svelte";

	let { event, tabindex }: { event: ScheduledTask; tabindex: number } =
		$props();
	let lessThan15Min = $derived(
		differenceInMinutes(event.ends_at, event.starts_at) < 16,
	);
	let loading = $state(false);

	let isDone = event.status === "Done";
	let isTask = event.event_type === "Task";
	let isReminder = event.event_type === "Reminder";

	async function toggleStatus(e: Event) {
		e.stopPropagation();
		loading = true;
		await commands.setVeventStatus(event.id, isDone ? "inprogress" : "done");
		invalidateAll();
		loading = false;
	}

	let classes = $derived([
		"event-card",
		`event-card-${event.event_type.toLowerCase()}`,
		lessThan15Min ? "text-xs" : "text-sm",
	]);

	let [importance, load, urgency] = $derived.by(() =>
		isTask || isReminder ? [event.importance, event.load, event.urgency] : [],
	);

	function openEvent() {
		eventUpserter.state = EventUpsert.Updating(event);
	}
</script>

<div class="size-full p-0.5 group">
	<div
		role="button"
		{tabindex}
		class:text-gray-400={isDone}
		class:text-white={!isDone}
		class={classes}
		onclick={openEvent}
		onkeypress={(event) => {
			if (event.key === "Enter") {
				openEvent();
			}
		}}
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
		{#if lessThan15Min && (isTask || isReminder) && !isDone}
			{importanceToString(importance, "|")}
			{urgencyToString(urgency, "|")}
			{loadToString(load)}
		{/if}
		<div class="absolute top-1 right-1">
			<button
				disabled={loading}
				class={[
					`done-button glass-clickable opacity-0 group-hover:opacity-100`,
					{ loading, lessThan15Min },
				]}
				onclick={toggleStatus}
			>
				{#if isDone}
					<UnCheckTask />
				{:else}
					<CheckTask />
				{/if}
			</button>
		</div>
	</div>
</div>

<style lang="postcss">
	@reference "../../../app.css";

	.event-card {
		@apply px-2 py-1 rounded-2xl size-full box-border relative text-white;
		backdrop-filter: blur(10px);
	}

	.event-card-event {
		@apply glassy-shadow-teal-600;
	}

	.event-card-task {
		@apply glassy-shadow-task-600;
	}
	.event-card-reminder {
		@apply glassy-shadow-reminder-400;
	}

	.event-card.done-button.lessThan15Min {
		@apply text-nowrap overflow-ellipsis;
	}

	.done-button {
		@apply cursor-pointer transition-opacity duration-200;
		@apply rounded-full p-1;
	}
	.done-button :global(svg) {
		@apply size-4;
	}

	.done-button.lessThan15Min {
		@apply p-0.5;
	}

	.done-button.lessThan15Min :global(svg) {
		@apply size-3.5;
	}
</style>
