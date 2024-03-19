<script>
	import Badge from 'flowbite-svelte/Badge.svelte';
	import FloatingLabelInput from 'flowbite-svelte/FloatingLabelInput.svelte';
	import Button from 'flowbite-svelte/Button.svelte';

	import { format, isSameDay } from 'date-fns/fp';
	import { formatISO, formatRelative } from 'date-fns';
	import { parseTaskText } from '$lib/parser';

	/** @type {string} */
	export let content = 'something today at 10 until 13 @event #mine #project !! ^^^';

	let taskText = content + ''; // Duplicate to avoid chainging the prop
	const today = new Date();
	/** @type {ReturnType<parseTaskText>}*/
	let taskInfo;
	$: {
		taskInfo = parseTaskText(taskText, today);
	}

	/** @type {import('./$types').Snapshot<string>} */
	export const snapshot = {
		capture: () => taskText,
		restore: (value) => taskText = value
	};

	// TODO use current date?
	let dateStr = formatISO(today);
</script>

<div>
	<h5 class="mb-2 text-2xl font-bold tracking-tight text-gray-900 dark:text-white">New Event</h5>
	<div>
		<form method="POST" class="flex-1" >
			<FloatingLabelInput
				id="original_text"
				name="originalText"
				type="text"
				label="Type your event"
				required
				bind:value={taskText}
			>
				Type your task
			</FloatingLabelInput>
			<input type="text" bind:value={dateStr} readonly class="hidden">
			<div class="mt-3 flex">
				<div class="flex-1">
					<div>
						{taskInfo.status.toUpperCase()}: {taskInfo.title}
					</div>
					<div class="mt-2">
						<Badge large color="green">@{taskInfo.type}</Badge>
						{#if taskInfo.importance !== 0}
							<Badge large color="red">
								{new Array(Math.abs(taskInfo.importance))
									.fill(taskInfo.importance > 0 ? '!' : '?')
									.join('')}
							</Badge>
						{/if}
						{#if taskInfo.urgency > 0}
							<Badge large color="yellow">
								{new Array(Math.abs(taskInfo.urgency)).fill('^').join('')}
							</Badge>
						{/if}
						{#if taskInfo.date}
							<Badge color="indigo" large>
								{formatRelative(taskInfo.date, new Date(), { weekStartsOn: 1 })}
								{#if taskInfo.endDate}
									until
									{#if !isSameDay(taskInfo.endDate, taskInfo.date)}
										{format('dd MMM yy HH:mm', taskInfo.endDate)}
									{/if}
								{/if}
							</Badge>
						{/if}
						{#each taskInfo.tag as t}
							<Badge class="mr-1" color="pink" large>#{t}</Badge>
						{/each}
					</div>
				</div>
				<div class="flex-0">
					<Button type="submit">Create</Button>
				</div>
			</div>
		</form>
	</div>
</div>
