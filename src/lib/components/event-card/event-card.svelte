<script lang="ts">
	import {
		importanceToString,
		loadToString,
		urgencyToString,
		type ParsedEvent,
	} from "$lib/util.js";
	import { differenceInMinutes } from "date-fns";
	import { ArrowsRepeatOutline } from "flowbite-svelte-icons";
	// @ts-expect-error iconify
	import CheckTask from "~icons/solar/bill-check-linear";
	// @ts-expect-error iconify
	import UnCheckTask from "~icons/solar/bill-cross-linear";
	import { commands } from "../../../bindings";
	import { invalidateAll } from "$app/navigation";

	let { event }: { event: ParsedEvent } = $props();
	let moreThan15Min = $derived(
		differenceInMinutes(event.starts_at, event.ends_at) < 16,
	);
	let loading = $state(false);

	let isDone = event.status === "Done";
	let isTask = event.event_type === "Task";
	let isReminder = event.event_type === "Reminder";

	async function toggleStatus() {
		loading = true;
		await commands.setVeventStatus(event.id, isDone ? "inprogress" : "done");
		invalidateAll();
		loading = false;
	}

	let classes = $derived([
		"event-card",
		`event-card-${event.event_type.toLowerCase()}`,
		moreThan15Min ? "text-xs" : "text-[0.6rem]",
	]);

	let [importance, load, urgency] = $derived.by(() =>
		isTask || isReminder ? [event.importance, event.load, event.urgency] : [],
	);
</script>

<div class="size-full p-2 group">
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
		{#if moreThan15Min && (isTask || isReminder) && !isDone}
			{importanceToString(importance, "|")}
			{urgencyToString(urgency, "|")}
			{loadToString(load)}
		{/if}
		<div class="absolute top-1 right-1">
			<button
				disabled={loading}
				class={[
					`done-button glass-clickable opacity-0 group-hover:opacity-100`,
					{ loading },
				]}
				onclick={toggleStatus}
			>
				{#if isDone}
					<UnCheckTask size="xs"></UnCheckTask>
				{:else}
					<CheckTask size="xs"></CheckTask>
				{/if}
			</button>
		</div>
	</div>
</div>

<style lang="postcss">
	@reference "../../../app.css";

	.event-card {
		@apply p-1 pr-5 rounded-lg glassy-shadow size-full box-border relative text-white;
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

	.done-button {
		@apply cursor-pointer opacity-0 group-hover:opacity-100 transition-opacity duration-200;
		@apply rounded-full p-1;
	}
	.done-hover {
	}
</style>
