<script>
	import FloatingLabelInput from 'flowbite-svelte/FloatingLabelInput.svelte';
	import Button from 'flowbite-svelte/Button.svelte';
	import Popover from 'flowbite-svelte/Popover.svelte';
	import { Accordion, AccordionItem, Badge, Helper } from 'flowbite-svelte';

	import { format, isSameDay } from 'date-fns/fp';
	import { formatDuration, formatISO, formatRelative } from 'date-fns';
	import { parseTaskText, unparseTaskText } from '$lib/parser';
	import { importanceToString, loadToString, urgencyToString } from '$lib/util';

	import * as pkg from 'rrule';
	// @ts-expect-error - see https://github.com/jkbrzt/rrule/issues/548
	const { RRule } = pkg.default || pkg;

	/** @type {import('./$types').PageData}*/
	export let data;
	/** @type {import('./$types').ActionData} */
	export let form;

	const originalText = form?.originalText ?? (data.event ? unparseTaskText(data.event) : ''); // Duplicate to avoid chainging the prop
	let taskText = originalText;
	const today = new Date();
	/** @type {boolean} */
	let editting;
	/** @type {ReturnType<parseTaskText>}*/
	let taskInfo;
	$: {
		taskInfo = parseTaskText(taskText, today);
		editting = typeof data.event !== 'undefined';
	}

	// TODO use current date?
	let dateStr = formatISO(today);
</script>

<div class="flex h-full w-full flex-row items-center align-middle">
	<div class="flex-1 p-10">
		<h5 class="mb-2 text-2xl font-bold tracking-tight text-gray-900 dark:text-white">
			{editting ? 'Edit Event' : 'New Event'}
		</h5>
		<p>For creating tasks we use text rather than controls.</p>
		<Accordion flush>
			<AccordionItem>
				<span slot="header">Explanation</span>
				<p class="mb-2 text-gray-600 dark:text-gray-400">
					<span class="mr-1 font-semibold">Title:</span>
					The title is what remains after all other modifers have been parsed.
				</p>
				<p class="mb-2 text-gray-600 dark:text-gray-400">
					<span class="mr-1 font-semibold">Type:</span>To type the task, prefix with <code>@</code>, can be <code>Block</code>,
					<code>Event</code>, <code>Task</code> or <code>Remainder</code>
				</p>
				<p class="mb-2 text-gray-600 dark:text-gray-400">
				<span class="mr-1 font-semibold">Status:</span>
					The status of task can be indicated by prefixing with <code>%</code>, can be <code>back</code>,
					<code>todo</code>, <code>doing</code> or <code>done</code>
				<p class="mb-2 text-gray-600 dark:text-gray-400">
					<span class="mr-1 font-semibold">Dates:</span>To set dates of a task/event. It will be parsed from the text in parenthesis. After the
					date you can add a <code>|</code> and a recurrence pattern (check rrule.js).
				</p>
				<p class="mb-2 text-gray-600 dark:text-gray-400">
					Example: Go shopping (at 21 until 23 | every monday) -> Date: today 9pm until 23:00 [every week on Monday]
				</p>
				<p class="mb-2 text-gray-600 dark:text-gray-400">
					<span class="mr-1 font-semibold">Tags:</span>To set tags just add <code>#</code> before a word.
				</p>
				<p class="mb-2 text-gray-600 dark:text-gray-400">
					<span class="mr-1 font-semibold">Importance:</span>
					To set the importance of the task use <code>?</code> for less import or <code>!</code>
					for more important ones. You can add up to 3 of either. <code>??</code> means
					-2, and <code>!</code> means +1
				</p>
				<p class="mb-2 text-gray-600 dark:text-gray-400">
					<span class="mr-1 font-semibold">Urgency:</span>
					Urgency is dictated by <code>^</code>. You can add up to 3.
				</p>
				<p class="mb-2 text-gray-600 dark:text-gray-400">
					<span class="mr-1 font-semibold">Load:</span>
					Load is dictated by <code>^</code>. You can add up to 3.
				</p>
				<p class="mb-2 text-gray-600 dark:text-gray-400">
					<span class="mr-1 font-semibold">Alarms:</span>
					Alarms are set using a <code>*</code> followed by ISO 860 duration. 
					For example *PT1H30M -> is alert me 1 hour 30 min before. And *P1DT1H is a day and an hour before
				</p>
			</AccordionItem>
		</Accordion>

		<div>
			<form method="POST" class="mt-2 flex-1">
				<FloatingLabelInput
					id="original_text"
					name="originalText"
					type="text"
					label="Type your event"
					required
					autofocus
					bind:value={taskText}
				>
					Type your task
				</FloatingLabelInput>
				{#if form?.errors}
					<Helper class="pt-2 text-red-950">
						Please fix the following errors
						<ul class="list-disc">
							{#each form.errors as e}
								<li>{e}</li>
							{/each}
						</ul>
					</Helper>
				{/if}
				<div class="mt-3 flex">
					<div class="flex-1">
						<div class="mb-1 flex text-lg">
							<p class="mr-1 font-semibold">Title:</p>
							<p>{taskInfo.title}</p>
						</div>
						{#if taskInfo.tag.length > 0}
							<div class="mb-1 flex">
								<p class="mr-1 font-semibold">Tags: {' '}</p>
								{#each taskInfo.tag as t (t)}
									<Badge rounded class="mr-1" color="purple">{t}</Badge>
								{/each}
							</div>
						{/if}
						<div class="mt-3 flex">
							<div class="flex-0 px-1">
								<div class="mb-1 border-b border-solid border-green-500">Type</div>
								<div>
									@{taskInfo.type}
								</div>
							</div>
							<div class="flex-0 px-1">
								<div class="mb-1 border-b border-solid border-gray-400">Status</div>
								<div>
									%{taskInfo.status}
								</div>
							</div>
							{#if taskInfo.date}
								<div class="flex-0 px-1">
									<div class="mb-1 border-b border-solid border-gray-400">From</div>
									<div>
										{format('dd/MM/yy HH:mm', taskInfo.date)}
									</div>
								</div>
							{/if}
							{#if taskInfo.endDate}
								<div class="flex-0 px-1">
									<div class="mb-1 border-b border-solid border-gray-400">Until</div>
									<div>
										{#if taskInfo.date && !isSameDay(taskInfo.endDate, taskInfo.date)}
											{format('dd/MM/yy HH:mm', taskInfo.endDate)}
										{:else}
											{format('HH:mm', taskInfo.endDate)}
										{/if}
									</div>
								</div>
							{/if}
							{#if taskInfo.recur}
								<div class="flex-0 px-1">
									<div class="mb-1 border-b border-solid border-gray-400">Recur</div>
									<div>
										{RRule.fromString(taskInfo.recur).toText()}
									</div>
								</div>
							{/if}
							{#if taskInfo.alarms.length > 0}
								<div class="flex-0 px-1">
									<div class="mb-1 border-b border-solid border-gray-400">Alarms</div>
									{#each taskInfo.alarms as alarm}
										{formatDuration({...alarm.duration}, { format: ['days', 'hours', 'minutes']})} before | 
									{/each}
									<div>
									</div>
								</div>
							{/if}
						</div>
						<div class="flex">
							{#if taskInfo.importance != 0}
								<div class="flex-0 px-1">
									<div class="mb-1 border-b border-solid border-gray-400">Importance</div>
									<div>
										{importanceToString(taskInfo.importance)}
									</div>
								</div>
							{/if}
							{#if taskInfo.urgency != 0}
								<div class="flex-0 px-1">
									<div class="mb-1 border-b border-solid border-gray-400">Urgency</div>
									<div>
  									{urgencyToString(taskInfo.urgency)}
									</div>
								</div>
							{/if}
							{#if taskInfo.load != 0}
								<div class="flex-0 px-1">
									<div class="mb-1 border-b border-solid border-gray-400">Load</div>
									<div>
									  {loadToString(taskInfo.load)}
									</div>
								</div>
							{/if}
						</div>
					</div>
					<div class="flex-0">
						<Button type="submit">{editting ? 'Update' : 'Create'}</Button>
					</div>
				</div>
			</form>
		</div>
	</div>
</div>
