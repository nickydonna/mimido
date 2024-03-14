<script>
	import { addMinutes, differenceInMinutes, parse } from 'date-fns/fp';
	/** @typedef {import('$lib/server/db/event.entity.js').TEventSchema} TEventSchema */

	/** @type {TEventSchema} */
	export let event;

	const height = (() => {
		if (!event.date || !event.time) return;

		const start = parse(event.date, 'HH:mm', event.time)

		const end = event.endDate && event.endTime
			? parse(event.endDate, 'HH:mm', event.endTime)
			: addMinutes(30, start);
		let size = differenceInMinutes(start, end) / 30;
		return (2 * size).toString() + "rem";
	})()
</script>

<div class="relative w-full h-8">
	<div class="absolute bg-amber-400 z-10 p-2" style:height>
		{event.title}
	</div>
</div>