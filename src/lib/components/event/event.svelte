<script>
	import { addMinutes, differenceInMinutes, parse } from 'date-fns/fp';
	import { EType } from '$lib/components/task-box/parser';

	/** @typedef {import('$lib/server/schemas/event.js').TEventSchema} TEventSchema */

	const ETypeClass = {
		[EType.BLOCK]: 'bg-blue-400',
		[EType.EVENT]: 'bg-amber-400',
		[EType.TASK]: 'bg-pink-400',
		[EType.REMINDER]: 'bg-red-400',
	}

	/** @type {TEventSchema} */
	export let event;
	/** @type {EType} */
	export let type;

	const height = (() => {
		if (!event.date || !event.hasStartTime) return;

		const end = event.endDate && event.hasEndTime
			? event.endDate
			: addMinutes(30, event.date);
		let size = differenceInMinutes(event.date, end) / 30;
		
		return (2 * size).toString() + "rem";
	})()
</script>

<div class="relative w-full h-8">
	<div class={`absolute z-10 w-full p-2 ${ETypeClass[type]}`} style:height>
		{event.title}
	</div>
</div>