<script>
	import { addMinutes, differenceInMinutes, parse } from 'date-fns/fp';
	/** @typedef {import('$lib/server/schemas/event.js').TEventSchema} TEventSchema */

	/** @type {TEventSchema} */
	export let event;

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
	<div class="absolute bg-amber-400 z-10 p-2" style:height>
		{event.title}
	</div>
</div>