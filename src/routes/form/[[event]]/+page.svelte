<script>
	import FloatingLabelInput from 'flowbite-svelte/FloatingLabelInput.svelte';
	import Button from 'flowbite-svelte/Button.svelte';
	import Popover from 'flowbite-svelte/Popover.svelte';

	import { format, isSameDay } from 'date-fns/fp';
	import { formatISO, formatRelative } from 'date-fns';
	import { parseTaskText, unparseTaskText } from '$lib/parser';

	import * as pkg from 'rrule';
	import { Accordion, AccordionItem, Badge, Helper } from 'flowbite-svelte';
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

	/** @type {import('./$types').Snapshot<string>} */
	export const snapshot = {
		capture: () => taskText,
		restore: (value) => (taskText = value)
	};

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
						</div>
						<div class="flex">
							{#if taskInfo.importance != 0}
								<div class="flex-0 px-1">
									<div class="mb-1 border-b border-solid border-gray-400">Importance</div>
									<div>
										{['Sub-Zero', 'Very Low', 'Low', '', 'Mid', 'High', 'Very High'][
											taskInfo.importance + 3
										]}
									</div>
								</div>
							{/if}
							{#if taskInfo.urgency != 0}
								<div class="flex-0 px-1">
									<div class="mb-1 border-b border-solid border-gray-400">Urgency</div>
									<div>
										{['Soon', 'Next Up', 'Why are you not doing it'][taskInfo.urgency - 1]}
									</div>
								</div>
							{/if}
							{#if taskInfo.load != 0}
								<div class="flex-0 px-1">
									<div class="mb-1 border-b border-solid border-gray-400">Load</div>
									<div>
										{['Mid', 'Hard', 'Fat Rolling'][taskInfo.load - 1]}
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

<!-- <div class="flex-1">
						
							
							<div class="px-4 py-2 sm:grid sm:grid-cols-3 sm:gap-4 sm:px-0" id="status-info">
								<dt class="text-sm font-medium leading-6 text-gray-900">Status (%)</dt>
								<dd class="mt-1 text-sm leading-6 text-gray-700 sm:col-span-2 sm:mt-0">
									{taskInfo.status}
									<Popover placement="bottom" triggeredBy="#status-info" title="The status of the task">
										The status of task, prefix with <code>%</code>, can be <code>back</code>,
										<code>todo</code>, <code>doing</code> or <code>done</code>
									</Popover>
								</dd>
							</div>
							<div class="px-4 py-2 sm:grid sm:grid-cols-3 sm:gap-4 sm:px-0" id="tag-info">
								<dt class="text-sm font-medium leading-6 text-gray-900">Tags (#)</dt>
								<dd class="mt-1 text-sm leading-6 text-gray-700 sm:col-span-2 sm:mt-0">
									{taskInfo.tag.join(', ')}
									<Popover placement="bottom" triggeredBy="#tag-info" title="The tags of the task">
										The tags for this task, prefix with <code>#</code>, there can be many of these.
									</Popover>
								</dd>
							</div>
							<div class="px-4 py-2 sm:grid sm:grid-cols-3 sm:gap-4 sm:px-0" id="importance-info">
								<dt class="text-sm font-medium leading-6 text-gray-900">Importance (? or !)</dt>
								<dd class="mt-1 text-sm leading-6 text-gray-700 sm:col-span-2 sm:mt-0">
									{taskInfo.importance ?? 0}
									<Popover placement="bottom" triggeredBy="#importance-info" title="The importance of the task">
										How important is this task, use <code>?</code> for less import or <code>!</code>
										for more important ones. You can add up to 3 of either. <code>??</code> means
										-2, and <code>!</code> means +1
									</Popover>
								</dd>
							</div>
							<div class="px-4 py-2 sm:grid sm:grid-cols-3 sm:gap-4 sm:px-0" id="urgency-info">
								<dt class="text-sm font-medium leading-6 text-gray-900">Urgency (^)</dt>
								<dd class="mt-1 text-sm leading-6 text-gray-700 sm:col-span-2 sm:mt-0">
									{taskInfo.urgency ?? 0}
									<Popover placement="bottom" triggeredBy="#urgency-info" title="The urgency of the task">
										How urgent the task is, use <code>^</code>. You can add up to 3.
									</Popover>
								</dd>
							</div>
							<div class="px-4 py-2 sm:grid sm:grid-cols-3 sm:gap-4 sm:px-0" id="load-info">
								<dt class="text-sm font-medium leading-6 text-gray-900">Load ($)</dt>
								<dd class="mt-1 text-sm leading-6 text-gray-700 sm:col-span-2 sm:mt-0">
									{taskInfo.load ?? 0}
									<Popover placement="bottom" triggeredBy="#load-info" title="The load of the task">
										How hard is the task, use <code>$</code>. You can add up to 3.
									</Popover>
								</dd>
							</div>
						</dl>
					</div> -->
